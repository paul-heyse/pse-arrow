# Test execution system architecture

Current source baseline `05e6a2b8a`, 2026-10-23. The holistic follow-up
revalidates each primary finding below; diagnoses are *Interface-checked*, not new test results.
See the [integrated architectural account](architecture-and-assurance.md),
[baseline and revalidation outcomes](README.md#revalidation-outcomes), and the
[test-assessment companion](../design_principles/core/test-assessment.md).

## <a id="dta-022"></a>DTA-022 — Runner consumers independently interpret selection, profiles and fixture identity

**Principles:** AP-02, AP-03, AP-04, DP-01, DP-14; companion §6.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`test_run.rust_report_command` recognizes long profile options while the Arrow composer accepts
`-P`; the report overlay can target a different profile from execution. `validation.native_report_config`
also rewrites TOML with different behavior. Managed selection and composer accepted-option
tables recognize different filter forms. Fixture identity is reconstructed in Rust
`testing::selected_fixture_identity` and Python `test_resources.selected_test`.

The composed requirement is stronger than each parser accepting its local inputs: actual
selection, overlay and report must describe the same execution. This mismatch is source-visible;
no fresh CLI probe was run. Executable authentication and explicit refusal of unknown scope are
important safeguards. Replacing them with unauthenticated environment labels would lose assurance.

## <a id="dta-023"></a>DTA-023 — Preparation and qualification selections require name-keyed maintenance

**Principles:** AP-01, AP-05, DP-24; companion §6, §8.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`RUST_FIXTURE_TESTS`, nextest requirements and `validation_scope.comprehensive` maintain test-name
knowledge outside the bodies. The dead python-runner recipe selects removed tests. Renaming a
Clarabel absence control can drop it from a selected qualification union without making the
remaining selection empty. This creates cross-owner maintenance for an ordinary test change.

**Corrected composition.** Supervised unit-package does prepare the special fixtures:
select → run_rust → observer_rust → ordinary_rust_capture → _rust_capture →
prepare_rust_fixtures, with rust_fixture_bindings forwarding them. The earlier claim that this
route always fails those tests was false. The absence gate is also not the control's only possible
route: a suitable ordinary non-Pardiso selection can run it. Keep the name-dependent campaign
correspondence issue, rather than treating all manual controls as routeless.

## <a id="dta-024"></a>DTA-024 — Reuse identities omit consequential generator inputs and test-tool identity

**Principles:** AP-02, DP-09, DP-15, DP-22; companion §6, §8.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`bootstrap.sh` installs current cargo-tool releases; no required nextest version is declared,
although integration code depends on tool behavior. Per-gate `validation_scope.input_identity`
and retained artifact checks do not capture a test-tool identity. This is a compatibility/reuse
premise gap, not evidence that the installed tools presently misbehave.

The generator omission is concrete: generation scope excludes packages, while
`xtask/src/codegen/physical.rs::load` reads `packages/reference/fixture-projection.toml` and its
selected package inputs. Explicit unchanged-input reuse can therefore miss a consumed edit.
Whole-run source capture still records it. The earlier statement that current provenance captures
rustc -Vv was not established and is removed; source, host and Cargo metadata are distinct facts.
No current tool version or historical run was requalified here.

## <a id="dta-026"></a>DTA-026 — Reciprocal lifecycle dependencies and several roots obstruct local test-system reasoning

**Principles:** AP-01, AP-03, AP-06, DP-17; companion §6, §8.

**Follow-up disposition:** Reframed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

Reciprocal imports connect test_run/native_tests/validation, validation_receipts/test_resources,
and test_resources/surreal_server. The server supervisor interprets registry lifecycle fields;
registration and removal call service readiness/admission owners. Explaining one composed test
run requires following these dependencies, and multiple entry roots deliver the concrete
guarantee differences in DTA-015 and option/report mismatch in DTA-022.

**Corrected footprint.** Registry ResourceRecord is a TypedDict, not an untyped record. Host
admission, native process drain and environment ownership serve general workloads; their whole
code/test volume cannot be charged to test-only support. Pass-through callers already return an
integer, so fixing return/verdict policy does not demonstrably require editing all of them.
Multiple roots or cycles alone do not prove a violation: lifecycle safety genuinely needs
coordination, and extensive actual-process tests challenge it. Retain demonstrated local-reasoning
and composed-contract issues, without prescribing file splitting or a new runner framework.
