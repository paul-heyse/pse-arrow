# Evidence routes, isolation and execution

Current source baseline `05e6a2b8a`, 2026-10-23. The holistic follow-up
revalidates each primary finding below; diagnoses are *Interface-checked*, not new test results.
See the [integrated architectural account](architecture-and-assurance.md),
[baseline and revalidation outcomes](README.md#revalidation-outcomes), and the
[test-assessment companion](../design_principles/core/test-assessment.md).

## <a id="dta-012"></a>DTA-012 — Managed command exit and reconciled assurance verdict can disagree

**Principles:** AP-05, DP-21; companion §6.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`scripts/test_run.py::execute_selected` composes report/selection errors into a failed check,
attempts terminal freezing, then returns the child's integer status. `select.py`, `python_tests.py`
and native callers preserve that return. The self-test
`test_actual_child_status_survives_missing_report_reconciliation` deliberately expects child
exit zero with incomplete evidence. An exit-only consumer therefore cannot interpret zero as
the complete reconciled assurance verdict; the terminal/report must also be read.

Child-status preservation is deliberate and the evidence records missing outcomes truthfully.
This is an outcome-interface distinction, not proof the composer fabricates a pass. Contrary to
the earlier finding, deferred post-terminal reclamation errors are printed without changing the
already-frozen verdict; only report/composition/finalization failures belong in that account.
No policy change or test rerun is selected here.

## <a id="dta-013"></a>DTA-013 — Some controls are inactive or outside declared campaigns; manual routes are not absent

**Principles:** AP-05, DP-22, DP-23, DP-24; companion §4, §6.

**Follow-up disposition:** Reframed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`crates/pse-py/Cargo.toml` disables its Rust test/doctest harness while
`src/inspection/errors.rs` retains unit tests. `scripts/setup_report.py::main` selects files by
literal `unittest.TestCase`, leaving the pytest-only context-overlap file outside that discovery
and default pytest testpaths. Standard `native_root_features` does not select every optional
capability such as compiled-evaluator/native-profiles. These are specific selection/coverage
boundaries, not a proof that every auxiliary control must be part of ordinary qualification.

**Corrected consumer evidence.** Generic unit routes accept explicit features.
`docs/plans/39d-qualification-and-plan38-cutover.md` records manual `canonical_signal_control`
invocation; `docs/plans/30c-test-ownership-and-evidence-retention.md` records
`plan30_context_overlap_run`. Source-only searches excluding those instructions incorrectly
called them unreferenced. Those historical instructions establish route discoverability, not
current qualification, mandatory scope or authority to revive retired campaigns. Python boundary
tests also cover diagnostic envelopes despite the inactive Rust harness.

An external-server-restart rendezvous still lacks a producer in the inspected current scripts,
recipes and workflows. Other optional controls were not each exhaustively traced through history.
Keep inactive bodies, optional manual controls and omitted required campaign selections distinct.
The portable-reconstruction evidence limit in the holistic account is not proof that its public
component API or ordinary scientific workflow has no valid route.

## <a id="dta-014"></a>DTA-014 — Optional CLI qualification has no separate executed outcome in enclosing Rust API tests

**Principles:** AP-05, DP-22, DP-23; companion §4, §6.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`canonical_explanations.rs::qualify_operator` returns when its selected environment inputs are
absent. Its graph/candidate callers still make substantive API, pagination, query and observation
assertions before that optional CLI step. Their pass is real evidence for those API properties,
but does not separately report whether the extra CLI qualification ran.

`scripts/tests/plan50_database_fixture.py` supplies the inputs for an explicit selected route.
Opt-in unittest controls also report skip reasons. The earlier description of all these bodies
as silent or vacuous passes was too broad. The remaining issue is the distinct scope of optional
evidence, not a requirement that each ordinary API test start the CLI or missing prerequisite
become a failure when that additional campaign was not selected.

## <a id="dta-015"></a>DTA-015 — Sanctioned routes compose different test-specific lifecycle and scheduling guarantees

**Principles:** AP-03, AP-05, AP-06, DP-19, DP-20; companion §6, §8.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`test_resources.register` initially pins an incomplete resource; `finalize` requires an exact
invocation and authentic outcome before deriving disposable pass. Fixture Drop records drain,
not success. Raw Arrow-composed Cargo routes lack that test-outcome composition, so resources
remain conservatively retained. General host admission and process supervision still apply.

Root pytest collection adds `canonical-owner`, but ordinary `native_tests.python_command`
selects loadgroup/allocation width while pyproject defaults to worksteal/sixteen workers.
`affected.py` composes validation then executes Cargo through pse-env, bypassing
`rust_test_environment`. Its stated exclusions do not explain canonical-test feature selection.
Thus changing the entry route can change test-specific grouping, association, stack or cleanup
semantics independently of the body's responsibility. No observed race, OOM or missing general
resource protection is inferred from these differences.

## <a id="dta-016"></a>DTA-016 — Scheduling requirements depend on names without complete correspondence checks

**Principles:** AP-05, DP-24, DP-20; companion §6, §8.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`.config/nextest.toml` expresses exclusive/timing requirements through regexes and exact names.
`test_plan30_routing.py::test_exclusive_store_controls_reserve_actual_scheduler_width` checks
one known selection rather than correspondence of all requirements with current tests. A rename,
split or new test can therefore change scheduling without changing its actual fixture needs.

The known stale durability term is in the database-only group whose cap equals default width;
it does not establish current interference. Per-database modules/buckets are contrary evidence
to some alleged server-wide conflicts. The expiry test's short-clock/pool behavior needs
particular care, but its actual cross-database interference was not established.

## <a id="dta-017"></a>DTA-017 — A stale validation declaration is checked while some routes bypass actual leaf verification

**Principles:** AP-04, AP-05, DP-01, DP-23; companion §6.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

Cargo workspace `validate-features`, a relation-manifest comment and governance
`every_crate_registered.rs::validate_features_exist` retain the universal relation-feature story.
Consumed scope-aware enforcement instead belongs to `arrow_validation.resolve`, including
normal/build/dev target/host leaves. Maturin's develop command and some codegen commands pass
feature names directly, and Python admission observes compiled feature names rather than the
complete composer leaf proof.

The self-test protects an unused declaration, while actual assurance depends on another owner.
The pse-py feature does forward validation explicitly. No invalid Arrow-consuming closure was
demonstrated. The finding concerns split authority and verification routes, not observed unsafe
execution or permission to remove force-validation.

## <a id="dta-018"></a>DTA-018 — moved

DTA-018 moved in the third pass (2026-10-23) to
[Efficiency-oriented design principles](execution-efficiency.md#dta-018). It is reframed
there as a complete integration lifecycle per fixture, correcting the earlier attribution
of database-local guarantees to invocation-wide premises
(H2, H12, DP-10). Its identity is unchanged.

## <a id="dta-019"></a>DTA-019 — Some shared test doubles and declared dev dependencies have no external consumer

**Principles:** AP-01, AP-03, DP-16; companion §2, §3.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

Across inspected Rust source in crates/tests/benches/xtask, pse-testkit's factory, NativeFixture
and capture have real consumers, while FaultStore/FaultPlan/CountingStore/execution/introspection
are consumed only by their own tests. Runtime/lifecycle/conformance pse-testkit and some proptest
dev declarations also exceed source use. This is bounded maintenance and dependency surface
without current adoption, not evidence that a useful abstraction can never earn its scope.

The wrappers' own test independently exercises one-shot faults, conditional writes and consumed
bytes. Existing product fault controls may serve their obligations by other means; unused doubles
do not prove absent fault coverage. A later correction should assess actual consumer needs rather
than forcing adoption merely because a shared helper already exists.

## <a id="dta-025"></a>DTA-025 — Fixture infrastructure failure is reported through Rust test failure and can fan out

**Principles:** AP-06, DP-19, DP-21; companion §6, §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`pse-operations/src/testing.rs::resource_bridge/canonical_fixture_options/canonical_fixture_with_profile`
returns infrastructure failures as CanonicalError::Configuration. Test consumers unwrap these
errors and the terminal composer records failed Rust cases, while pytest distinguishes setup
errors. Setup executes inside the Rust test body, which supplies no separate setup-phase result;
this is a composed diagnostic limitation, not nextest misclassifying an existing structured setup
outcome. A shared outage can obscure the product contract that was never reached.

Readiness and registration can repeat for remaining selected tests, amplifying diagnosis and
infrastructure work when fail-fast is disabled. Finite clocks, startup joining, early require_store
and specific error messages limit that behavior. The old remaining-tests/width/timeout formula
did not follow from source and is removed. No outage duration, repeated failure count or observed
cleanup-pressure attribution is claimed.
