# Doubles, seams, composition and contract coupling

Current source baseline `05e6a2b8a`, 2026-10-23. The holistic follow-up
revalidates each primary finding below; diagnoses are *Interface-checked*, not new test results.
See the [integrated architectural account](architecture-and-assurance.md),
[baseline and revalidation outcomes](README.md#revalidation-outcomes), and the
[test-assessment companion](../design_principles/core/test-assessment.md).

## <a id="dta-008"></a>DTA-008 — Static routing policy tests are not evidence of a defective test-only composition

**Principles:** AP-02, AP-06, DP-23; companion §1, §5.

**Follow-up disposition:** Withdrawn, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

The earlier misalignment diagnosis is withdrawn. `routing.rs::policy_eligibility_for_test` and
`policy_decision_for_test` explicitly scope themselves to static class/rank rules. They consume
the actual adapter table, reuse `admit` and `decide_assessed`, and deliberately supply Ready
conditions. Their narrower scope does not promise to exercise the full contextual assessment.
`routing/contextual_tests.rs` separately exercises production decision/eligibility with concrete
structure and representation inputs. `execution/tests.rs::stub_backend_routes_through_adapter_table`
does use the adapter table even though it does not prove all contextual readiness obligations.

Those controls are complementary responsibilities. No concrete supported composition obligation
missing from their union was established. A test-only seam is not itself a second production
path or a demand to initialize all production context in a policy unit test. Keep this retired
identity for lineage; withdrawal reflects corrected judgment, not an implementation repair.

## <a id="dta-009"></a>DTA-009 — Reuse observations depend on private query names and some lack positive controls

**Principles:** AP-02, DP-09, DP-23; companion §4, §5.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`workspace/modeling/granular_tests.rs::observed/count`, `reuse_tests.rs` and `document_tests.rs`
inspect formatted Salsa execution events for private function-name substrings. A query rename
can turn an observed counter into zero; equality-only controls such as
`relocating_unchanged_declarations_preserves_type_and_member_memos` can then continue passing.
Exact counts also bind tests to internal query decomposition. This is an observability/contract
coupling problem, not a claim of wrong incremental results.

Many reuse tests assert a positive count first, and `assert_fresh` compares incremental output
with independently rebuilt output. Those controls preserve semantic assurance. The footprint
outside the inspected event observers remains unestablished; no new Salsa capability claim or
upgrade experiment was needed for this source-level diagnosis.

## <a id="dta-010"></a>DTA-010 — Implicit fixture construction duplicates preparation mechanics across crates

**Principles:** AP-01, AP-02, AP-06; companion §2, §5.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`pse-math/src/implicit.rs::source_body_for_test/finish_body_for_test/residual_bodies_for_test`
and `pse-backend-native/src/implicit.rs` repeat real prepared-source binding and compilation
mechanics for handcrafted numerical fixtures. The production builder releases prepared source;
the helpers retain it for source attribution. The copies already differ in cancellation and
residual-body construction details, so a preparation-contract change requires examining both.

The demonstrated issue is maintenance coupling of fixture mechanics. Different local needs can
justify differences, and drift is not proved harmless or undetectable by their consumers. These
helpers do not duplicate the scientific expected-value oracle. Neither a test-support feature nor
a new shared crate is selected; a remedy must retain actual source bindings and independent cases.

## <a id="dta-011"></a>DTA-011 — Fixture lifetime ownership changes the canonical-store path exercised by some tests

**Principles:** AP-02, AP-06, DP-19; companion §5, §6.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`pse-operations/src/canonical.rs::CanonicalStore` has a feature/test-gated `fixture_lifetime`.
`seal_final_fixture_owner` consults its strong count, and protection cleanup retains it before
the fixture can drain. These are meaningful extra lifetime paths, so fixture-backed drop tests
alone do not establish the identical ownership behavior of a library without that field.

**Corrected scope.** Dormant counters, failure probes and controlled pauses throughout the
runtime are not automatically defects merely because they are cfg-gated. They provide useful
deterministic race/failure evidence and ordinarily fall through to the real operation when unset.
The Python production extension omits the test-support feature; actual Python close/reopen and
retained-reader journeys therefore complement Rust fixture tests. The direct transport control
with `fixture_lifetime: None` is further contrary evidence. No suite-wide untested production
shape or real lifetime bug was established. Preserve deterministic controls while distinguishing
the particular lifetime behaviors each build and fixture actually exercises.

## <a id="dta-027"></a>DTA-027 — Reference-source dependencies are maintained in several test assembly lists

**Principles:** AP-01, AP-03, AP-04, DP-01; companion §2, §5.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`tests/fixtures/reference-source-fixture.rs::SOURCES/reference_sources/rows` embeds an explicit
document union, injects a data-only dependency, parses under a synthetic source identity and
rewrites selected data lineage. Compiler/modeling tests also maintain private document closures.
An imported source dependency change consequently requires coordinated edits to these test lists;
the same reference can be consumed through different assembly paths.

**Scope correction.** This is deliberately bounded declaration/scientific preparation. Its
synthetic data and omitted external transport need not exercise manifest resolution, which the
production loader and `tests/support/plan14.rs` exercise separately. It is not automatically a
second writable production authority or an invalid test merely because it bypasses the manifest.
The demonstrated issue is repeated maintenance and dependency knowledge, with manual lineage
adaptation a possible drift point. Missing required declarations fail loudly. A smaller sufficient
closure and the semantic equivalence of an alternative loader remain to be established.
Compilation coupling and repeated preparation keep their separate homes in DTA-035 and DTA-030.

## <a id="dta-028"></a>DTA-028 — Free-text modeling refusal contracts couple some negative tests to wording

**Principles:** AP-02, AP-05, DP-02, DP-21; companion §4.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`pse-modeling/src/lib.rs::ModelingError::Contract` identifies a declaration but carries its
violated condition as `message: String`; `invalid()` constructs that representation and compiler
errors preserve it. Consumers therefore sometimes inspect diagnostic substrings, or accept any
error, instead of asserting a stable semantic refusal distinction. Rewording can break the former;
the latter may accept a different cause. This is test-side evidence about a production boundary,
not authorization to replace the error model here.

Domain, realization and objective refusals are typed and have contrasting typed tests.
Single-mutation admitted controls and extra event assertions also constrain some text/bare-error
tests; their mere use is not enough to condemn them. The prior broad counts were not refreshed.
The existing broad account's title search revealed no same primary finding; it was not refreshed.
