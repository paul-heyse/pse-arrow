# Oracles, acceptance and assertions

**Interface-checked · 2026-10-27 · source `87f2eb9050ae24976956628c16e0e7376190be7a`.** This page
holds the primary findings on scientific expectations, references, assertions and test technique.
The [collection baseline](README.md#baseline-and-method) states the standard and limits; the
[integrated account](architecture-and-assurance.md) explains how these findings interact with
execution and ownership. No test, generator or conformance run was executed for this refresh.

Closure records below replace the earlier diagnoses whose correction current source establishes.
The prior text is preserved at the
[2026-10-24 publication](https://github.com/paul-heyse/pse-arrow/blob/cbf45ccc933b66192bd7ed5b2a77176e39d46c2e/docs/design_review/test-audit/oracles-and-acceptance.md).
[Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions) owns disposition.

## <a id="dta-049"></a>DTA-049 — The authored scientific reference library has no route in ordinary or comprehensive qualification

**Principles:** PS-13, PS-03, PS-07, PS-08, PS-10, DP-22, DP-23, AP-05; companion §7, §8.
**Status:** new, 2026-10-27.

**Claim at risk.** The model library's scientific fidelity: authored unit and property models agree
with their declared IDAES, teqp, FeOS and analytic oracles; recycle cases converge from defaults and
restore specifications on failure; real-model derivatives are consistent.

**Structural cause.** The reference set (`packages/reference/conformance.toml`, about 123 fixtures with
oracle provenance across 30 packages) is executed as a set only by the product conformance CLI through
`justfile::seed-conformance`. That route sits outside both the test catalogues and the qualification
scope:

- `just assessment-list` (`scripts.validation --list`) lists the recipe neither among its checks
  nor in its `exclusions` map (`scripts/validation_scope.py::EXCLUSIONS`), which names parity,
  performance, powerset and others. §24.2 outcomes decline whole-library qualification, but the
  assessment report does not carry that declaration.
- The only manifest test, `python/pse/tests/test_modeling_kernel.py::test_conformance_runs_the_declared_reference_set`,
  loads the manifest and runs a synthetic one-fixture package.
- Fewer than a fifth of the seed-data fixture names appear anywhere in `crates/`, `tests/` or
  `python/pse/tests`; ordinary Rust and Python suites use their own small fixtures
  (`tests/fixtures/plan14`, synthetic packages).

**Consequence.** An edit to a seed package, to specialization/compiler lowering or to an authored
thermodynamic method can break oracle agreement (for example the BT_PR flash comparison) or recycle
convergence while `just test`, `just native-test`, `just py-test` and `just assessment` stay green.
No full pass is recorded at all. The last full run
(Plan 28e, 2026-10-11, retired to Git history) failed nine and left eleven of 118 fixtures
inconclusive; Plan 29 records another two-hour failed run; Plan 53a's selected seed runs had failed
and inconclusive results before its receiving fix. No later full run is recorded. A comprehensive
assessment report therefore does not show that the product's central scientific claim was not
exercised, or that its last exercise failed.

**Strongest legitimacy argument.** The run is expensive (hours, native solvers and a native
extension), §24.2 outcomes explicitly decline whole-library claims, and the schema/relations guide
names `seed-conformance` as the run. These justify keeping a full sweep out of every loop. They do not
justify an omission the assessment report does not declare, a recorded failure with no owner tracking
its repair, or the absence of any representative slice (for example one fixture per unit/property
family) between manual sweeps.

**Footprint and uncertainty.** The whole authored model library's oracle fixtures. Seed *packages*
are executed elsewhere: the seed-transport family (`tests/support/plan14.rs::seed_package_on`,
`python/pse/tests/test_plan14_acceptance.py`) loads `packages/reference/seed-data`, but exercises the
packages, not their oracle fixtures. Whether the 28e failures have since been repaired is unknown. Related:
[DTA-050](evidence-routes-and-execution.md#dta-050) (activation correspondence),
[DTA-054](#dta-054) (reference staleness).

## <a id="dta-054"></a>DTA-054 — Reference provenance binds frozen outputs but not generator inputs or expectation-level derivation

**Principles:** PS-13, AP-04, DP-21, DP-22; companion §4, §7.
**Status:** new, 2026-10-27; extends the residual of [DTA-003](#dta-003).

**Claim at risk.** "The frozen value reproduces from the authored inputs by an independent method"
(`packages/reference/seed-data/sources.md`), and published oracle attribution.

**Structural cause.**

- The 53a generators read frozen *outputs* from the fixture (`scripts/scientific_reference.py::frozen_number`)
  but hard-code every *input*: `scripts/pfr_reference.py::INPUTS` (also imported by
  `cstr_controller_reference.py`) and `decay_fit_reference.py::OBSERVATIONS`. Input correspondence is
  recorded only as hashes in ignored `build/plan53a-reference-results/*.json` and never compared.
  `just plan53a-reference` belongs to no qualification selection, and the generators are absent from
  the scientific-input closure behind `affected` (`tests/fixtures/scientific-inputs.toml`).
- The plan14, FeOS, SRK and NRTL generators have the converse half-binding: they read authored banks
  but overwrite goldens, with no compare mode; acceptance rests on reviewing the Git diff.
- Provenance is attached per test (`test … oracle references.X`) and published as
  `oracle_source_id` by `workflow/modeling/conformance.rs`. In `bt-ideal.pse` the 358/378 K regime
  fixtures cite `idaes_test_btideal` (a 368 K state-block observation), and
  `mixer_unequal_pressure_minimum` cites `analytic_stream_identity` while its values come from an
  energy balance and a minimum-pressure rule. The DTA-002 witness restates premises as literals
  (`slack_lower = 5.`, z = 0.5, `1e-8`).

**Consequence.** An edit to shared `saponification.pse` (reaction enthalpy or Arrhenius factor) can
move the PFR outlet slightly; the native consumer still passes against unchanged frozen values within
its tolerance, and the generator still passes against its stale inputs. The provenance claim silently
becomes false and nothing reports staleness. Attribution readers can take an analytic or regime
expectation for an IDAES observation, which is the misreading that originally produced DTA-002.

**Strongest legitimacy argument.** Generators must not import product code. Reading authored *data*
is not a product import, and the generators already read outputs that way; comparing recorded input
hashes would also give a staleness signal. Coarse per-test attribution is honest where comments
explain derivation, but not where a consumer treats `oracle_source_id` as the derivation.

**Footprint.** Three 53a generators, the plan14/FeOS/SRK/NRTL freeze generators and authored oracle
attribution. This is a validity/staleness concern, not a demand for claim-to-evidence traceability.

## <a id="dta-056"></a>DTA-056 — Universal identity and pruning claims are challenged by curated examples where invariance relations and existing oracles would discriminate

**Principles:** AP-06, DP-13, DP-23, PS-10, H4; companion §1, §4.
**Status:** new, 2026-10-27.

**Claims at risk.** Value identity, row tokens, literal round trips and schema fingerprints are
deterministic and layout-independent for every type `admit_type` accepts; pruning never loses rows.

**Structural cause.**

- `pse-columnar/src/native_value.rs`, `row_token.rs`, `pse-schema/src/literal.rs`,
  `fingerprint.rs` and `pse-relations/src/identity.rs` show each arm by one curated example, mostly
  on unsliced flat arrays (`native_value/consolidation_unit.rs`,
  `row_token/consolidation_unit.rs::dictionary_reordering_and_slicing_do_not_change_value_tokens`,
  `literal/consolidation_unit.rs::field_directed_roundtrips_preserve_native_layout_and_metadata`).
  Oracle-free relations exist: slicing commutes with tokens, concatenation composes, dictionary
  packing is invisible, literals round-trip. The already pinned `proptest =1.11.0` can supply
  layout strategies. The Arrow crate's `util/data_gen.rs` sits behind its `test_utils` feature,
  which under workspace feature unification (ADR-0122) would change Arrow's feature set for every
  member, so it is not a free option.
- `tests/engine/tests/pushdown_vs_unpruned.rs` uses a sound multiset oracle with a negative control,
  but one 36-row batch and 11 queries: no range predicates, no `!=`/`NOT` over nullable columns and
  no block or file boundaries. `pse-runtime/src/workflow/result_snapshot.rs::prune_members` feeds
  repository-encoded statistics into DataFusion's `PruningPredicate`, yet is tested by a two-block
  fixture under `canonical-tests` and never through that oracle.
- The DSL AST property test is compiled out of every executing route
  ([DTA-050](evidence-routes-and-execution.md#dta-050)); the "fails without panicking" parser claim
  rests on about ten hostile examples.

**Consequence.** An arm that mishandles a sliced union, a dictionary of a nested type or nested
nulls produces unstable identity (spurious cache misses) or collisions (wrong result reuse). A
statistics-binding mismatch that over-prunes on a range or null predicate returns fewer rows. The
current examples would not detect either.

**Strongest legitimacy argument and limits.** Curated cases target known-hard points and arms read
through Arrow accessors that handle offsets. No concrete defect was found; the gap is discrimination
for a universal claim. Round trips alone share encoder and decoder; invariance relations do not.

**Contrast.** Structural analysis is exhaustively enumerated against brute-force witnesses
(`pse-structural/src/incidence_tests.rs`), identity has frozen vectors, and implicit-product
association uses proptest with a demonstrated shrinking fault. These show the techniques are known
and used well elsewhere.

## <a id="dta-058"></a>DTA-058 — Layer and governance descriptions claim more than their checks discriminate

**Principles:** DP-22, AP-05, AP-04; companion §4.
**Status:** new, 2026-10-27; moderate-low.

- §24.1 says `tests/lifecycle` establishes guarded canonical admission, interruption, exact retained
  reads and result memory budgets; the package holds memory-budget tests only
  (`query_memory_budget.rs`, `result_memory_budget.rs`). `tests/engine` holds two test files against a
  description covering provider hierarchy, plan codec, relational expansion and unified sources.
- AGENTS.md requires every `pub enum *Error` to implement `miette::Diagnostic` "with a blueprint
  §23.2 code". §23.2 scopes `tests/governance/tests/error_taxonomy.rs` to implementation presence,
  relying on the compiler for declared code variants, so a *derived* `Diagnostic` with no `code`
  passes every check.
- Eight of the fourteen `compile_fail` doctests in `crates/` pin no error code, so an unrelated
  compile error passes.
- `.config/nextest.toml` says tests that own server-wide resources or depend on sub-second timing
  "take all sixteen slots"; `test_requirements.scheduling` grants that only to `exclusion`
  requirements, in practice the `external-restart` peer
  ([DTA-016](evidence-routes-and-execution.md#dta-016)).
- `python/pse/tests/test_modeling_kernel.py::test_conformance_runs_the_declared_reference_set`
  checks that the manifest loads and runs a synthetic one-fixture package; its name claims the
  reference set runs ([DTA-049](#dta-049)).

**Consequence.** Reviewers and agents choosing evidence by layer or name can believe interruption and
retained-read semantics, exclusive scheduling or reference-set execution are covered where they are
not; an error enum without a code passes. This is evidence accuracy, not proof that the behaviors lack
tests elsewhere. ADR-0160 moved contracts to owning crates; some descriptions did not follow.

## Closure records and residuals for earlier findings

### <a id="dta-001"></a>DTA-001 — Result-relative accuracy checks do not independently establish the allowance projected by that route

**Refresh:** withdrawn as an unestablished indictment; nothing changed. Plan 53's "disproved" label
overstates it: the earlier account found an unproven boundary, and `test_sensitivity_parity.py` still
derives its allowance from `runtime.resolved_numerics`. Complementary resolver controls
(`pse-math/src/numerics.rs::engineering_accuracy_uses_frozen_scale_and_equivalent_unit_magnitudes`,
`tests/support/plan14.rs::resolved_physical_allowance`) remain. Revisit only for a concrete escaped
projection fault.

### <a id="dta-002"></a>DTA-002 — Historical phase-reference comparisons do not distinguish every phase regime

**Refresh:** closure established. `tests/conformance/src/acceptance/complementarity_flash.rs::{absent_phase_bound, assert_phase_regime, phase_regime_envelope_rejects_interior_and_missing_slack}`
derives the regime bound from the frozen Wagner bank and actual row budgets, with a wrong-interior
control. Restated premise literals belong to [DTA-054](#dta-054).

### <a id="dta-003"></a>DTA-003 — Three computed references lack a committed regeneration path

**Refresh:** closure established for provenance and reproduction
(`scripts/{pfr,cstr_controller,decay_fit}_reference.py`, `scripts/scientific_reference.py`,
`scripts/tests/test_scientific_references.py`). Input binding and staleness remain as
[DTA-054](#dta-054).

**Plan-state discovery (not an audit finding).** Ignored reports under
`build/plan53a/controller-after-receiving-fix/` were written at 2026-10-24 10:50:12, 45 seconds after
handoff commit `25b75d338`, which records the simultaneous-controller consumer as still running.
Plan 53's pending block itself names these six `seed*.arrow` reports and asks for identity to be
verified before A04/I10/G01 change. Read with pyarrow, they show the fixture `inconclusive` on the
scientific preparation deadline with a `resource_limit` finding. If their identity is confirmed,
A04/I10 and G01 hold an inconclusive result rather than a pending one. Plan 53 owns that
reconciliation; this account infers nothing beyond the reports.

### <a id="dta-004"></a>DTA-004 — The HiGHS mapping test does not detect loss of a declared status arm

**Refresh:** closure established. `crates/pse-backend-native/src/highs.rs::full_status_mapping_preserves_limits_and_ambiguity`
checks every code plus out-of-range values as literal code/name/category/assurance tuples.

### <a id="dta-005"></a>DTA-005 — The integrated PR jet check shares primitive derivatives with the path it checks

**Refresh:** closure established. `pse-runtime/src/workflow/modeling/pr_jacobian_tests.rs::{independent_pr_coordinate_jet, integrated_pr_two_root_mixed_hessian_has_independent_analytic_oracle}`
derives the implicit-function jet independently and checks roots, permutations, storage and all
Hessian cells.

### <a id="dta-006"></a>DTA-006 — Opaque-input rejection checks can accept the wrong refusal

**Refresh:** closure established. `crates/pse-admission/tests/quantity_relations.rs::mixture_normalization_preserves_differences_and_requires_actual_fraction_contracts`
asserts the intended inference rule per mutation beside an admitted control. Remaining bare
`is_err()` sites each sit beside an admitted contrast and are not material.

### <a id="dta-007"></a>DTA-007 — Some numerical helpers lack shape and non-finite discrimination

**Refresh:** closure established. The three comparators check finiteness (and extent/sample time for
dynamics) with wrong-value controls. Minor: the dynamics `close` ignores `Sample::{mode, integrals}`;
its consumers compare integrators.

### <a id="dta-020"></a>DTA-020 — Published KKT-derived results lack a nonlinear-constraint reference case in the inspected portfolio

**Refresh:** closure established on the native canonical route.
`sensitivity_tests.rs::nonlinear_constraint_curvature_sensitivity_and_value_hessian_reopen` and
`fitting/covariance_tests.rs::nonlinear_state_constraint_exact_and_gauss_newton_covariance_reopen`
discriminate exact from approximate curvature through durable reopen.

### <a id="dta-021"></a>DTA-021 — Some default model variants lack discriminating reference scenarios

**Refresh:** closure established for the two named variants
(`physical_nlp.rs::authored_countercurrent_and_unequal_pressure_variants`). Library-wide variant
discrimination depends on the route gap in [DTA-049](#dta-049).
