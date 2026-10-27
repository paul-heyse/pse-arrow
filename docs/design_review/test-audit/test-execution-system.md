# Test execution system

**Interface-checked · 2026-10-27 · source `87f2eb9050ae24976956628c16e0e7376190be7a`.** Closure records
for requirement interpretation and composition. Current residual causes live in
[DTA-015/016](evidence-routes-and-execution.md#dta-016) and
[DTA-050](evidence-routes-and-execution.md#dta-050). Prior diagnoses are preserved at the
[2026-10-24 publication](https://github.com/paul-heyse/pse-arrow/blob/cbf45ccc933b66192bd7ed5b2a77176e39d46c2e/docs/design_review/test-audit/test-execution-system.md);
[Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions) owns disposition.

## <a id="dta-022"></a>DTA-022 — Runner consumers independently interpret selection, profiles and fixture identity

**Refresh:** closure established. `scripts/test_invocation.py::{rust, python}` is the single
interpretation; its `report_configuration` serves both `rust_report_command` and
`validation.native_report_config`. Rust `selected_fixture_identity` and Python `selected_test`
deliberately authenticate each other.

## <a id="dta-023"></a>DTA-023 — Preparation and qualification selections require name-keyed maintenance

**Refresh:** partially resolved. `validate_roles` catches a lost role inside a nonempty selection.
The remaining name-keyed maintenance has primary homes elsewhere: file-name capsule relevance in
[DTA-027](doubles-and-coupling.md#dta-027), and a dead recipe filter (despite the disposition's
"obsolete references removed") in [DTA-050](evidence-routes-and-execution.md#dta-050).

## <a id="dta-024"></a>DTA-024 — Reuse identities omit consequential generator inputs and test-tool identity

**Refresh:** closure established. `cargo_tools.selected_identity` (version and digest) is rechecked in
`native_tests.prepared_rust_inventory`; `validation_scope.INPUT_SCOPES["generation"]` includes packages.

## <a id="dta-026"></a>DTA-026 — Reciprocal lifecycle dependencies and several roots obstruct local test-system reasoning

**Refresh:** closure established for its scoped target. Duplicated meaning moved into
`test_invocation`; performing cycles remain as lazy imports (`native_tests._rust_capture` imports
`test_run`; about twenty-five lazy imports in `native_tests`, twenty in `test_resources`). Plan 53 did
not target module-count cleanup, so this is not reopened. The broad account's
[DA-037](../audit/development-tooling.md#da-037) owns the wider tooling cycle.
