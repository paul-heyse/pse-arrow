---
title: "Plan70a: Selection and execution"
status: in-progress
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [design, validation, preparation]
review_sources:
  - docs/design_review/reviews/design_review_test-assurance-target_2026-11-08.md
  - docs/plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md
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

#### Analyze the admitted graph and compare complete unit records

For [I01](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i01),
validate the reached union of Cargo unit indices once per unchanged admission. Propagate Arrow
reachability back through those edges to determine which roots need explicit validation.
This removes overlapping root walks and carries the reached set through unchanged arguments.
Reject unknown package IDs and malformed indices. Keep host/build, target and selected-root dev
contexts distinct. A feature or diagnostic-isolation change requires a newly admitted Cargo
graph, identical root identity and validation of the final relevant Arrow leaves. Cargo owns
the edges and features. PSE owns the decision and must refuse a missing propagation route.

For [I02](../plan_improvement_reviews/plan70-repository-test-efficiency-2026-11-08.md#i02),
the correctness owner supplies one order-independent multiset comparison of complete
`CompilerUnit` records. Replace equal-length membership checks in discovery, managed workflow,
prepared xtask, check capture and workflow comparison. An ordered comparison or counted records
preserve repeated unit multiplicities without introducing a digest identity. Keep checks whose
contract includes order, and keep separate argument, capability, exit, source observation and
custody checks. [E03](70e-preparation-and-verification-scope.md#e03) consumes this comparison at
its scoped receivers. [Managed workflow](../../crates/pse-host/src/managed/workflow.rs) ·
[Prepared xtask](../../crates/pse-host/src/prepared_xtask.rs) ·
[Check capture](../../crates/pse-host/src/test_workflow/execute/runner.rs) ·
[Workflow comparison](../../crates/pse-host/src/test_workflow/mod.rs)

Compare old and new root decisions on a shared graph with a pure root and Arrow through build
and dev edges. Include malformed references, graph-changing feature admission and actual focused
and broad pinned selections. Count visited edges as structural evidence. Accept a permutation
of identical records and refuse equal-length multiplicity substitution and changed host/target,
feature or mode. Exercise actual receiving refusal alongside the local comparison (DP-09, DP-15,
H10, H18–H19).

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

Use [E05](70e-preparation-and-verification-scope.md#e05) for focused harness and command changes
that alter the compiled target population.

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [A01](#a01) | complete | Current Cargo adapter and tool pin | Actual nextest identity and scoped unit-graph and doctest controls. Shared graph analysis preserves validation decisions. Complete unit comparison reaches its migrated receivers and rejects substitution. |
| [A02](#a02) | complete | A01 runner identity and existing managed result receiver | Exact selected requirement correspondence, explicit verdict and settlement, and named missing/wrong/unmarked controls. |
| [A03](#a03) | complete | A02 requirement ownership for any enforcement, B02 drain contract for receiving checks | Record the resource owner and observed contention boundary. Where needed enforce it and prove independent overlap and cancellation behavior. |
| [A04](#a04) | complete | A01/A02 route semantics, D02 reference admission for parity, E01 for source-consuming routes, B02 for deadline-sensitive qualification | Each named specialist route selects the intended case and prerequisites. Remove superseded invocation and stale route descriptions. |

## Execution checkpoint

Reconciled on 2026-11-08 against the combined implementation, Cargo coordination, and resumed
native qualification. The remaining specialist dependencies are recorded below.

**Implemented.** A01 analyzes the reached unit graph once for each admitted argument set
and compares complete unit records at the existing receiving boundaries. The generic
Nextest route verifies the checkout pin on an invocation-private captured executable and
checks those bytes again before dispatch. Managed discovery uses the same pin interpretation.

**Tested.** The reference walk agreed on pure, Arrow and workspace root decisions. In the
128-root workspace selection, the prior walks visited 186,960 edges and the reached-union
analysis visited 8,852. These are structural counts. The default managed
`pse-host:correctness` selection passed 14 `native_correctness_` controls, and the targeted
`pse-columnar` doctest selection passed all three cases against a zero-failure baseline.
`ae069ea7d` adds a real pinned Cargo and rustdoc control with a local Arrow-shaped dev leaf.
The ordinary library check succeeded, the omitted validation leaf refused before execution,
and the corrected feature route executed the doctest. The exact managed rerun
`01a1bba8-2119-7764-bd18-45a59a88b7f1` passed with zero failures. The focused host batch
also passed the prepared xtask receiving substitution control.
The combined managed host run passed 47 of 48 cases. The target-owned settlement test
failed on temporary-directory permissions. Its directory setup was corrected in `ca1eaa011`.
The exact failed-selection rerun `01a1bba4-7c9f-7469-a6a1-180a5a455e2a` verified that correction
with zero failures and complete settlement. See [B03](70b-fixtures-and-process-ownership.md#b03)
for the physical receiving controls.

**Implemented.** A02 associates prerequisites with exact cases and rejects a stale declared
scope or incompatible performing owners before fixture preparation. A native listing can
enumerate a skipped suite for declaration validation. Only the original listing authorizes
execution. A typed setup refusal from a selected native service target stays distinct from
an assertion failure and from physical settlement. The Park and foreground helpers are
child-only entries with explicit parent markers. The later default managed host selection
passed 11 queue and nested workflow controls with zero failures. That run exercised selection,
setup refusal and exact rerun on isolated Cargo fixtures. B03's live receiving checks establish
the original target-owned drain and the separate assertion verdict. The default Nextest Park
and foreground parent controls reached their intended phases, with helper entries kept outside
the unmarked assurance population.

**Interface-checked.** A03 traced the named scientific group to Nextest scheduling within
one invocation. Fresh borrowed runs construct their own private server; their child cases
use isolated UUID databases on that server. Target-owned lifecycle cases construct or reopen
their own private service state. The qualification expiry case saturates its own server's
controller pool and uses the store group within that run to keep competing cases out.
**Implemented.** The native service integration target has a rendezvous control for two
independent invocations on private servers. The pair `01a1bbaf-d7cc-7614-9877-a4e452a1770e`
and `01a1bbb0-1e35-73e5-9d58-fce065dc2062` passed with reciprocal rendezvous, distinct
services and complete settlement. The live native selection also passed the existing owner
checks for a foreign listener collision and original-attempt cancellation after admission closed.
The detached run `01a1bbe8-f280-738b-a12b-ef813ec7c51f` was cancelled after its ready witness.
The managed service disappeared, but the original target had only a start record and no terminal
workflow frame. Settlement remained unknown and all five captured executables remained in
original custody. The worker-owner control also retained admission through unknown reap and
released it after actual settlement. These selected effects use their existing owners and per-run
Nextest scheduling. Cargo output admission in
`82f578fa6` coordinates compilation. This inquiry concerns native effects after build and fixture
preparation.
[Nextest policy](../../.config/nextest.toml) ·
[Borrowing owner](../../crates/pse-operations/src/native_fixture.rs) ·
[Service owner](../../crates/pse-operations/src/native_service.rs)

**Implemented.** A04 selects the locked reference interpreter, separates reference-only,
solver and native-contract controls, and restores the direct pinned Miri route. The Study
qualification cases require the corresponding companion receipt separately from production
module borrowing. The default managed arbitrary AST selection passed its one case with
`pse-authoring/arbitrary`, and `just reference-unit` passed 15 solver-free controls.
`just reference-solver-unit` passed four locked Ipopt reference controls. `8aebd0544`
separates the large logical-description stress case from the focused local selection.
The resumed `just scientific-replay-miri` route passed all three selected cases and the
separate managed native stress case passed in `01a1bbc0-e520-7622-b465-2058d87e5e80`, both with
zero failures. With the standard companion, fault rollback and pool expiry passed. `7dea6f83c`
made the cancelled-apply test's read-only observer tolerate a typed live transaction conflict
under its original deadline. The exact four-controller rerun `01a1bbdc-c186-75e3-84c0-40a6199c4a90`
passed and settled. A production receipt presented as the companion refused before effects.
Fresh long-hold preparation succeeded and the eight-controller capacity case passed and settled
in `01a1bbdb-39a6-771e-8c7e-8e18afd00dd4`.
[Operator routes](../dev/validation-assessment.md#individual-commands)

The retained native reference contracts refused current source-closure validation. The fresh
managed `pse-runtime --lib` producer with Ipopt, KINSOL, and root isolation then passed five
selected cases with zero failures in `01a1bbf2-f862-72a4-8926-4bbbb8f0d3de`, and its managed
wait confirmed settlement. `just reference-contract-unit` passed all five independent receiver
tests against those fresh bytes. This establishes the selected producer and receiver routes.
The configured exchanger population and its scientific qualification remain with [Plan66l](66l-required-population-and-assembled-acceptance.md#packages)
on its own schedule.

A04 now uses E01's selected authored closure in ordinary compiler and modeling tests. The
capsule-free selection passed 20 tests with zero failures under the normal force-validated
correctness profile. The staged receipt cases retain their explicit capture route.

D02's four isolated producers observed the required performing solver stages in the locked
parity environment. The fresh managed ordinary producer passed 17 cases, including the
five binary comparisons, with zero failures and complete settlement. The independent
receiver passed six tests against those fresh contracts. `just parity` passed all four
selected native exchanges with zero failures and zero skips in the prepared, locked
reference environment. The parity recipe defaults to the local Nextest profile so
the native harness and reference child have their own deadline and settlement windows under
the profile's finite whole-run limit. Select build-bearing checks sequentially in the shared
checkout, or combine compatible selectors under one Cargo invocation.
