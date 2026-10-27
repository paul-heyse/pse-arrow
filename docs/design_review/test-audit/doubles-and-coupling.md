# Support ownership, doubles and coupling

**Interface-checked · 2026-10-27 · source `87f2eb9050ae24976956628c16e0e7376190be7a`.** This page holds
the primary findings on where test support and evidence live, how tests couple to production, and the
seams production carries for tests. See the [collection baseline](README.md#baseline-and-method) and
[integrated account](architecture-and-assurance.md). Prior diagnoses are preserved at the
[2026-10-24 publication](https://github.com/paul-heyse/pse-arrow/blob/cbf45ccc933b66192bd7ed5b2a77176e39d46c2e/docs/design_review/test-audit/doubles-and-coupling.md);
[Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions) owns disposition.

## <a id="dta-051"></a>DTA-051 — Test evidence and support are placed by creation history rather than enduring responsibility

**Principles:** AP-01, AP-03, AP-05, DP-17, DP-22, DP-24, H26, H27; companion §2, §3, §6.
**Status:** new, 2026-10-27.

**Claim at risk.** A retained control is current, runnable and protects a stated claim; tests and
support can be found, changed and retired by the responsibility they serve. The repository's own
rules require it: AGENTS.md forbids retaining a path "as evidence", ADR-0160 asks contracts to describe
responsibility boundaries rather than historical tests, and ADR-0096 retires completed plans.

**Structural cause.** A control's route, claim and owner are recorded in the plan or packet that
commissioned it, and support is created inside whichever plan driver or test module needed it first.
The retirement preflight in `docs/dev/documentation.md` finds executables that consume a plan
*document*, not executables whose only *route* is that plan. Retiring a plan therefore silently
orphans its drivers, and enduring tests acquire imports from plan-era modules.

**Footprint (representative).**

- `scripts/tests` holds 63 non-collected driver modules (about 29k lines). Eleven have no route in any
  recipe, guide, architecture text or plan history, including `mcp_native_acceptance.py` (about 2,100
  lines), `mcp_isolation_acceptance.py`, `active_mixed_control.py`, `active_diagnostic_control.py`,
  `plan52_test_evidence_check.py`, `surreal_otlp_check.py`, `surreal_logging_check.py` and
  `grpc_frame_probe.py`/`grpc_frame_relay.py` (hand-coded HTTP/2 framing). About 25 more are routed
  only by completed plans (ten `telemetry_*_acceptance.py`, eight `plan52_*.py`, inquiry drivers).
- Enduring collected tests import plan drivers: `test_telemetry_metrics` from
  `telemetry_operational_acceptance`, `test_surreal_diagnostics` from `surreal_mx_check`,
  `test_surrealkit_maintenance` from `surrealkit_rollout_fixture_check`. Twenty-five tooling test
  modules import from other `test_*` modules; `surreal_server` lifecycle behavior is spread across
  about twenty modules, some plan-named (`test_plan30_lifecycle`, about 2,000 lines).
- Product tests and support carry plan identities: `tests/support/plan14.rs` (used by conformance
  acceptance and `benches/native_process.rs`), `tests/fixtures/plan14/`,
  `python/pse/tests/test_plan14_acceptance.py`, fifteen Rust tests named `plan38b_*`, `plan39_*`,
  `plan47f_*`, `plan30_*`, `PSE_PLAN30_*` variables in an enduring guide, and the
  `just plan53a-reference` recipe.
- Python scenario builders live in the test modules that first needed them:
  `test_native_workflow.{package_documents, declaration, physical_context}`,
  `test_modeling_kernel.{physical, on_primitives}` and private `_package` helpers are imported across
  about twenty files. The primitives dependency (package `5a5a…` at 1.0.0) is restated eight times
  and spliced with `manifest.replace("dependencies = []", PRIMITIVES)`, which silently does nothing if
  the fixture text drifts. Shared `physical_input_fixture.py` exists but is partly adopted.

**Consequence.** Refactors keep route-less drivers type-correct without running them (commits
`d7d0169af` and `159d63169` edited fourteen and fifteen drivers, including route-less ones), so readers
can believe they are current. Retiring 52p–52s would orphan ten more. Owners must find tests by plan
number; product-test refactors ripple into drivers with no route; a drifted manifest silently
produces a different scenario.

**Strongest legitimacy argument.** A manual route in a guide is real (the plan30 drivers in
`agent-environment.md`, capability inquiry drivers kept for the R-55 trigger); the execution rhythm
forbids running them mid-plan; sharing an independent oracle between acceptance and its unit test is
good practice. These justify routed drivers and shared oracles, not route-less drivers or plan
identity as the organizing key.

## <a id="dta-055"></a>DTA-055 — Production types carry test choreography and fixture seams across several vocabularies

**Principles:** AP-01, AP-02, DP-18, DP-21; companion §5.
**Status:** new, 2026-10-27; related to closed [DTA-011](#dta-011).

**Structural cause.** Deterministic interleaving control and fixture lifetime are threaded through
production struct fields, parameters and call-site arguments under four overlapping vocabularies:
`cfg(test)`, `test-support`, `canonical-tests` and conjunctions with solver features. Static counts show
hundreds of `cfg(test)`-family items outside test modules in `pse-runtime`, and over a hundred in
`pse-operations`. Examples: `workflow/shooting/window_team.rs` gates and buffer identities;
`workflow/fitting/experiments.rs` (`gate`, `wave_gate`, `fixed_affinity_for_test`);
`workflow/physical_cache.rs` admission pauses and counters; `math/staged.rs` with a `cfg(test)`
parameter and a `cfg(not(test))` twin; `workflow/durable.rs::FinishExportGate`. Test-only counters
(`result_files.rs::PREFLIGHT_CALLS`, `canonical/local.rs::CONSTRUCTION_DECODES`) sit beside the existing
production observation channel. A `canonical-tests` worker variant has a different `serve` signature.

Two seams change exercised behavior or the shipped surface:

- Under `test-support`, `CanonicalStore::fixture_lifetime` routes protected release to the fixture's
  executor (`FixtureLifetime::cleanup_executor`) and changes drop order, so Rust canonical tests do not
  exercise the production executor path; the complement comes from Python's support-absent closure
  and a few `fixture_lifetime: None` transport tests.
- `Runtime::fixture_connection_owners` and `CanonicalStore::drain_fixture_connection_before` are
  ungated `#[doc(hidden)] pub` items, surfaced as `pse-py::NativeRuntime::_drain_fixture_connections`
  in the production extension and guarded only by a `canonical_test_` database-name prefix.

**Consequence.** Readers of core concurrency code must filter test choreography; seam combinations
compile only in particular feature sets; the shipped API includes fixture operations guarded by
convention.

**Legitimacy and limits.** Deterministic cancellation and drain interleavings back truthful-outcome
claims and are hard to obtain otherwise. Instrumentation does not reach the shipped extension
(`test-support` is dev-only; `pse-py` has no test feature), apart from the doc-hidden drain. The
purpose is legitimate; the cost is the scattering, the duplicated counter channel and the shipped
fixture surface. A related lead: `scripts/tests/test_surreal_server.py` (about 12k lines, about 1,000
patch sites) mostly patches the supervisor module's own functions; its likely cause is the
production supervisor's internal structure, which the broad account treats under
[DA-037](../audit/development-tooling.md#da-037). Not raised separately here.

## <a id="dta-027"></a>DTA-027 — Reference-source dependencies are maintained in several test assembly lists

**Refresh:** partially resolved. The 19-family catalogue derives the closure from production
capture once (`xtask/src/scientific_inputs.rs`), a real improvement. Residuals:

- **Demand side is hand-declared in three places**: the catalogue's `consumers` crate list,
  `scripts/scientific_inputs.py::RUST_TEST_OWNERS` (private `pse-runtime` test-module prefixes) and
  the Python test-file name list in `prepare_python` (`test_studies.py`, `test_plan14_acceptance.py`, …). Readers (`tests/fixtures/scientific-inputs.rs::{package_documents, file_bytes}`,
  `python/pse/tests/_scientific_inputs.py`) accept any captured path regardless of declared family.
  A new `pse-runtime` test outside the listed prefixes that calls `actual_reference_recycle_fixture`
  gets no preparation and panics with a message that looks environmental.
- **Live readers bypass the capsule**: `pse-modeling/src/domain_schema.rs::schema`,
  `pr_jacobian_tests.rs`, `math/solves/derived_tests.rs`, `pse-admission/tests/quantity_relations.rs`
  (a non-declared consumer) and `benches/benches/modeling_preparation.rs` read `packages/reference`
  directly, without exact-byte identity or archive replay.
- **A text-based adaptation outside the catalogue**: `tests/fixtures/reference-source-fixture.rs::{reference_family_sources, rows}`
  injects an import with `replacen("package bt_ideal {", …)` and renames lineage
  `idaes_thermo.rpp4_pressure_oracles`; neither checks that it applied. Reformatting `bt-ideal.pse`
  breaks every bounded-reference consumer with an unrelated admission error; renaming the oracle
  dataset silently removes `pressure_bindings`, which no test asserts.

## <a id="dta-009"></a>DTA-009 — Reuse observations depend on private query names and some lack positive controls

**Refresh:** partially resolved. `pse-compiler/src/workspace/modeling/test_observation.rs::{Query, QueryObservations::count}`
centralizes spellings and refuses missing/truncated traces, but matching is a substring search over
salsa's `Debug` event output and only `Query::Parsed` has a positive control. `EffectiveMembers` and
`Projection` assertions are equalities that a rename satisfies with zero on both sides, so
`relocating_unchanged_declarations_preserves_type_and_member_memos` can pass vacuously for member memos.

## <a id="dta-010"></a>DTA-010 — Implicit fixture construction duplicates preparation mechanics across crates

**Refresh:** partially resolved. `SourceBody::prepare` absorbed the common finish step, but the two
residual-body copies remain and still differ: `pse-math/src/implicit.rs::residual_bodies_for_test`
uses optimization parameters and evaluation limits, `pse-backend-native/src/implicit.rs::test_residual_bodies`
uses defaults, and both differ from `pse-runtime/src/math/native_child.rs::compile_problem`.

## <a id="dta-028"></a>DTA-028 — Free-text modeling refusal contracts couple some negative tests to wording

**Refresh:** mechanism established, closure scope overstated. Source-owned `ModelingContractReason`
covers the migrated distinctions (about eighteen `invalid_reason` sites) while the large remainder of
`invalid(...)` sites construct `Unclassified`, and about a hundred `to_string().contains(` assertions
remain in modeling/compiler tests. Read as resolved for migrated distinctions only.

## <a id="dta-008"></a>DTA-008 — Static routing policy tests are not evidence of a defective test-only composition

Withdrawn 2026-10-23; never reused. Complementary static and contextual readiness tests remain.

## <a id="dta-011"></a>DTA-011 — Fixture lifetime ownership changes the canonical-store path exercised by some tests

**Refresh:** closure established as scoped: `canonical-tests` selects participation, `test-support`
selects instrumentation, and support-absent Python reader/drop journeys exercise the complementary
production path. The wider seam pattern is [DTA-055](#dta-055).
