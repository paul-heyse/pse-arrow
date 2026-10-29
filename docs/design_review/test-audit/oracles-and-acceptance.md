# Oracles, acceptance and assertions

**Interface-checked · refreshed 2026-10-29 · source `7bd341f0bd4a27ad629008abdff56ee2bcf7e9f3` plus
the working tree.** This page holds the primary findings on scientific expectations, references,
assertions and test technique. The [collection baseline](README.md#baseline-and-method) states the
standard, the included uncommitted changes and limits; the
[integrated account](architecture-and-assurance.md) explains how these findings interact with
execution and ownership. No test, generator, oracle or conformance run was executed.

Finding bodies after the new DTA-061/062/065/066/067 keep their 2026-10-27 evidence (source `87f2eb905`) unless the
refresh table says otherwise. Closure records replace earlier diagnoses whose correction current
source establishes; the prior text is preserved at the
[2026-10-24 publication](https://github.com/paul-heyse/pse-arrow/blob/cbf45ccc933b66192bd7ed5b2a77176e39d46c2e/docs/design_review/test-audit/oracles-and-acceptance.md).
[Plan 53](../../plans/53-test-architecture-target.md#finding-dispositions) owns disposition of
DTA-001 to DTA-058; DTA-061, 062 and 065–067 have no disposition owner yet.

## Refresh at `7bd341f0b` — 2026-10-29

Plan58 moved the four IDAES/SciPy parity families from `python/pse/parity` to native
`crates/pse-runtime/tests/reference_{degeneracy,covariance,sensitivity,pid}_exchange.rs`, which run
isolated oracle processes from `tests/reference/oracles/*.py`. The oracle side is stronger than
before (see [strengths](architecture-and-assurance.md#contrasting-strengths)). The authored reference
library and the generators are unchanged.

| Finding | Outcome | Current residual and decisive evidence |
|---|---|---|
| [DTA-049](#dta-049) | Still live on the Python route | `just seed-conformance` still runs `python -m pse.conformance` on a Python runtime (`justfile:842-845`). The native conformance consumer runs one synthetic pure package (`expect square(2)==4`, `native_conformance.rs:50-60`) and no `packages/reference` fixture. No full run is recorded after the failed 2026-10-11 run. 58f F05 and 58h H02 own the native consumer. |
| [DTA-054](#dta-054) | Live, unchanged | The 53a generators still hard-code `INPUTS`/`OBSERVATIONS` (`scripts/pfr_reference.py:24`, `decay_fit_reference.py:23`, `cstr_controller_reference.py:25`). Contrast: the new oracles refuse any case that differs from their literal copy (`tests/reference/oracles/degeneracy.py:112-127`). |
| [DTA-056](#dta-056) | Live | `result_pruning_tests.rs` was rewritten in the working tree and moved behind `native-study-tests` (`result_snapshot.rs:1392`), with the same curated two-block, equality-predicate technique and no unpruned oracle. |
| [DTA-058](#dta-058) | Live, extended | `python/pse/parity/tests` now holds only `test_00_preflight.py`, so `just parity` (`justfile:952-958`) runs no comparison. AGENTS.md ("`just parity` runs the IDAES comparisons"), the assessment exclusion (`scripts/validation_scope.py:337`), the command guide (`docs/dev/validation-assessment.md:57`) and the §24.1 layer table still route parity claims there. |
| [DTA-001](#dta-001) | Withdrawn; premise inherited | The native exchanges carry the same derivation with a concrete claim at risk: [DTA-061](#dta-061). |
| [DTA-020](#dta-020) | Established; route caveat | Its evidence is gated by `canonical-tests` and `native-solvers` (`sensitivity_tests.rs:436-438`); every managed canonical route currently refuses `--store` ([DTA-050](evidence-routes-and-execution.md#dta-050)). |
| [DTA-003](#dta-003) | Established, caveat | Generators unchanged; the A04 report identity remains open in Plan 53a. |
| DTA-002, 004–007, 021 | Established, unchanged | No diff to their decisive files since `87f2eb905`. |

## <a id="dta-061"></a>DTA-061 — Reference comparison allowances cross the oracle boundary from the candidate under test

**Principles:** PS-13, PS-10, AP-04, DP-23; companion §1, §3, §7.
**Status:** new, 2026-10-29. Lineage: revisits the premise of withdrawn [DTA-001](#dta-001) with a
concrete claim at risk; the deleted Python parity tests used the same derivation, so this is
inherited by the cutover, not introduced by it.

**Claim at risk.** 58f F04 and 58h H03 require the exchanges to "preserve independently derived
answers, tolerances" so that native tests cannot manufacture their own acceptance. The covariance
oracle names its allowances "frozen native budgets".

**Structural cause.** The acceptance allowance travels from the candidate to the oracle:

- covariance: `reference_covariance_exchange.rs:205` sets
  `frozen_estimate_allowances: report.estimate_allowances.clone()` from the same run, so the oracle's
  equality check (`tests/reference/oracles/covariance.py:259-262`) is self-referential; its estimate
  comparison then uses those allowances;
- sensitivity: the allowance is read from the candidate's production `resolved_numerics`
  (`reference_sensitivity_support/mod.rs:276`); the oracle accepts any finite positive value;
- wrong-candidate controls perturb values by multiples of the candidate's own allowance
  (`reference_sensitivity_exchange.rs:179-187`); no control inflates the allowance itself;
- `native_scientific_modes.rs:520-525` uses the same candidate-supplied allowances.

PID is bounded on the oracle side (`pid.py:130-134`), and matrix and degeneracy comparisons use fixed
relative tolerances, so the footprint is the estimate and derivative comparisons.

**Consequence.** A nominal or scaling defect, or a wrong configured fraction, that inflates the
engineering budget widens exactly the comparisons meant to detect numerical error, and every negative
control still passes. (A resolver that misapplies the configured fraction is caught:
`reference_covariance_support/mod.rs:259-261` asserts the fraction and row budget.)

**Evidence and uncertainty.** Established for the cited exchanges. Resolver tests
(`pse-math/src/numerics.rs:1146`, `tests/support/plan14.rs:279`) check the budget arithmetic but no
test bounds its magnitude against an independent declaration. No escaped defect was found; the cases
are unit-scale. **Counterevidence:** the engineering budget is a contractual accuracy promise, so
comparing against it is meaningful once its magnitude is independently bounded.

## <a id="dta-062"></a>DTA-062 — The reference-environment contract is restated in each oracle and has drifted

**Principles:** PS-13, DP-01, DP-21, AP-04, H5; companion §4, §7.
**Status:** new, 2026-10-29.

**Claim at risk.** Reference numbers come from the declared environment: the old preflight stated
that iteration-sensitive comparisons are valid only against the pinned Ipopt 3.14 series
(`python/pse/parity/tests/test_00_preflight.py:24-26`).

**Structural cause.** Oracle isolation (`-I`, which also implies `-P`) prevents a shared helper
module, so each oracle reimplements `environment()` and each Rust target copies `pins()`/`exchange()`
(`reference_*_exchange.rs:56-136`). The copies disagree: `degeneracy.py:154` requires Ipopt 3.14.x
although its answer comes from SCIP; `covariance.py:148-157` and `sensitivity.py:133-143` probe the
version without requiring the series; `pid.py:146-153` checks only presence; SCIP and PETSc versions
are not recorded.

**Consequence.** An unpinned Ipopt on `PATH` produces parmest and sIPOPT reference values that are
accepted, while only the Ipopt-independent degeneracy oracle refuses.

**Evidence and uncertainty.** Established for the four oracles and four Rust copies. Numerical
sensitivity of these cases to the Ipopt version is not established. **Counterevidence:** Python pins
come from the single `pyproject.toml` `parity` group, and each oracle verifies them; isolation itself
is sound and is a strength.

## <a id="dta-065"></a>DTA-065 — Historical-compatibility evidence for canonical migration is built by the current code it checks

**Principles:** AP-04, AP-02, DP-23, DP-24, PS-10; companion §1, §4.
**Status:** new, 2026-10-29.

**Claim at risk.** 58d: an original target-v2 migration request, recorded by v2-era code, still decodes,
fingerprints and settles as a target-v2 receipt under current code; a schema change cannot leave
existing stores unsupported unnoticed.

**Structural cause.** The frozen predecessor DDL is genuinely independent: literal SHA-256 constants
(`crates/pse-schema/src/catalog/substrate.rs:20-34`) are verified before every migration body and
fixture install (`canonical_migration.rs:983-1000`). The request and receipt side is not. The live
historical-request test serializes a *current* request and overwrites its target fields
(`crates/pse-operations/tests/native_service.rs:496-503`); unit tests check
`bytes(decode(bytes(x))) == bytes(x)` against today's serde layout (`canonical_migration.rs:1546-1581`,
`1654-1677`). No frozen v2-era request, receipt or fingerprint exists, and the fingerprint is
`sha256` of the current serialization (`:455`). The current target digest is a generated constant
(`generated/surreal.rs:18`), and `supported()` accepts v3 only at that value (`:327-333`), so no
literal pins it.

**Consequence.** A field reorder or new serialized field changes every historical fingerprint while
these tests pass. A generator change that moves the digest without a version bump makes existing v3
markers unsupported with no failing test: the current digest appears only in the generated constant. Migration fixtures also carry no domain records, so
the next record-shape migration has no data-bearing test to build on.

**Evidence and uncertainty.** Established for the request/receipt fingerprint path and the
current-digest tripwire. Today the v2-era struct is a field subset of the current one, so the gap is
latent. **Counterevidence:** frozen DDL literals; `deny_unknown_fields` on request and predecessor;
checkpoint verification tests mutate a test-built snapshot.

## <a id="dta-066"></a>DTA-066 — Migration fault injection is added per call site and leaves the final-edge recovery branches unstimulated

**Principles:** DP-19, DP-22, DP-23, PS-10, AP-06; companion §5, §6.
**Status:** new, 2026-10-29.

**Claim at risk.** 58d and its checkpoint: an interrupted migration settles truthfully. A lost
acknowledgement or failed readback on the final edge yields the original receipt or a typed
`SettlementUnresolved`, never a replay.

**Structural cause.** Each canonical owner adds its own task-local fault flags
(`CONTROL_ACK_LOST`, `PREPARED_READBACK_FAILED`, `EXECUTION_ACK_FAULT`, and so on, across about seven
modules). For migration, only the checkpoint acknowledgement loss is injectable
(`checkpoint_v2_lost_ack_for_test`). The final edge's write-failure-then-readback branch and its
readback-failure `SettlementUnresolved { owner: "schema-migration" }` branch
(`canonical_migration.rs:917-930`) and the checkpoint's own readback-failure branches (`:858-880`) have
no stimulus. No test interrupts the process during the final DDL transaction, the step 58d split
because a two-edge transaction conflicted on the engine.

**Consequence.** The "final acknowledgement loss" evidence exercises `reconcile_receipt` after a
successful apply (`native_service.rs:661-663`), not the recovery code inside `apply_receipt`. Progression
and control have both acknowledgement-loss and readback-failure stimuli; migration, the hardest to
repair in a persistent store, does not.

**Evidence and uncertainty.** Established for the migration final edge and checkpoint readback;
other owners' hook sets were not enumerated. **Counterevidence:** the checkpoint lost-acknowledgement
test is strong (real commit, discarded acknowledgement, physical restart, original bytes compared); the
in-flight writer is forced with a held transaction, not a sleep.

## <a id="dta-067"></a>DTA-067 — Native refusal assertions accept any error

**Principles:** AP-05, DP-21, DP-23; companion §4, §6.
**Status:** new, 2026-10-29. Lineage: the class of the closed [DTA-006](#dta-006). The production-side
cause, untyped refusals, has its primary home in [DTA-028](doubles-and-coupling.md#dta-028); this
finding owns the test-side discipline.

**Claim at risk.** Refusals happen at the intended boundary and for the intended reason: capability
vocabulary before effects, closed admission, stale handle, foreign module, competing claim.

**Structural cause.** Negative tests assert only that an operation failed, not which refusal occurred.
In `pse-host`, where refusals are untyped (`HostError::Ownership(String)`, [DTA-028](doubles-and-coupling.md#dta-028)),
the tests contain about 119 bare `is_err()` checks against 8 typed matches; exit 125 is the declared
boundary contract, so the gap is typing at the test level, not the CLI status. In the canonical tests a
typed helper exists (`assert_schema_refusal`, called 4 times) and `RefusalCode` is available, yet about
36 bare `is_err()` remain across `native_service.rs`, `progression_controls.rs` (including the
competing-claim loser, `:331`) and the publication controls.

**Consequence.** An unrelated failure satisfies the refusal. `tests/native.rs:15-28` checks that
`native prepare --capabilities ipopt` fails; `run_cli` (`native.rs:832-848`) validates vocabulary, then
connects to D-Bus and checks containment, so if `run_cli` stopped calling the vocabulary check, the test
would still pass on the containment error outside containment and, inside it, proceed toward real
preparation. (The vocabulary check itself is covered without I/O by
`providers_capability_dependencies_and_invalid_vocabulary`, `native/providers.rs:1353-1360`.) A transport, timeout or `Quiesced` error passes
as "the old module must still refuse v3 activation" (`native_service.rs:553-562`).

**Evidence and uncertainty.** Structural cause established; exposure established for the cited
sites and suspected for other bare checks. **Counterevidence:** manager-protocol tests match typed
variants; `tests/boundary.rs` pairs exit 125 with stderr prefix and absence of effects;
`tests/correctness.rs` pairs refusal with a sentinel effect.

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
