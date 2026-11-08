---
title: "Plan70d: Scientific references"
status: planned
date: 2026-11-08
adrs: []
doc_role: plan
doc_owner: docs/plans/70-test-assurance-architecture.md
doc_topics: [current-work, design, validation, numerical-execution]
review_sources:
  - docs/design_review/reviews/design_review_test-assurance-target_2026-11-08.md
---

# Scientific references

## Target and baseline

**Proposed.** Independent reference cases own their physical input identity, expected result and
agreement tolerance. The native result carries its own stopping, numerical quality and engineering
policy observations. Compare those policies separately from reference agreement. The performing
oracle attests the locked environment and actual numerical tools it used. One bounded process
contract protects the exchange while each oracle retains its scientific method and wire format.
[Coordinator](70-test-assurance-architecture.md) ·
[Review R04](../design_review/reviews/design_review_test-assurance-target_2026-11-08.md#r04)

**Interface-checked at `622261d8f`.** The covariance request copies the primary candidate's
allowances into its supposedly frozen reference acceptance input. Sensitivity uses candidate
allowances and scales for comparisons, including its independent sIPOPT-to-analytic check. Their
existing perturbation controls use the allowance under test, so a wrong value paired with an
inflated allowance is a distinct failure case. The ordinary typed exchange already consumes an
independently produced reference and bounds its Proto encoding and readback. Preserve its separate
original engineering context, physical coordinates and unit checks. The outstanding live-output
issue is shared with the legacy exchanges. Their diagnostic files can grow during execution and
legacy response reads lack the ordinary decoder's extent check.
[Covariance caller](../../crates/pse-runtime/tests/reference_covariance_exchange.rs) ·
[Covariance oracle](../../tests/reference/oracles/covariance.py) ·
[Sensitivity caller](../../crates/pse-runtime/tests/reference_sensitivity_exchange.rs) ·
[Sensitivity oracle](../../tests/reference/oracles/sensitivity.py) ·
[Ordinary reference exchange](../../crates/pse-runtime/src/workflow/modeling/ordinary_reference.rs)

## <a id="d01"></a>D01 — Give the independent case control of the agreement decision

For covariance and sensitivity, derive acceptance precision independently of any submitted
candidate. Put the justification beside the case and its quantities, in original physical
units or in a normalization whose scales are fixed by the case. Use independently specified
finite perturbations and absolute allowances appropriate to each derivative, value, increment
or prediction. The covariance matrix and its parameter order, units, convention and analytic
inverse-information check stay explicit. For sensitivity, independently check the analytic
nominal and perturbed solutions, the sIPOPT prediction, and the native prediction and physical
derivatives. Bound the native-to-oracle comparison using the two case-owned error allowances
where both results are already independently checked. Keep a separate assertion for the
candidate's reported engineering allowances and scaling policy. Remove the derivation of
`frozen_estimate_allowances` from the primary covariance report and candidate-dependent
acceptance limits in sensitivity when their replacements land.

Give each retained reference claim the case identity, physical inputs and operating conditions,
datum or unit where material, independently derived values or bounds, producer identity and
release it actually used. Preserve the genuine 368 K BTIdeal attribution at that state. For
the 358 K and 378 K nested and complementarity regime cases, bind observations at those inputs
where independently available or attribute the independent analytical phase bounds and
closure checks actually used. Apply the same distinction to the unequal-pressure mixer
energy/minimum-pressure expectation. Keep configuration and expected quantities separate from
the candidate calculation. For the legacy generators cited in DTA-054, make compare mode
validate the applicable authored inputs and retained reference values without overwriting
them. Keep explicit regeneration distinct. An input digest can identify a captured source,
while the compared input values and their case association establish correspondence.
[BTIdeal fixtures](../../packages/reference/seed-data/models/bt-ideal.pse) ·
[Reference provenance](../../packages/reference/data/references/models/references.pse) ·
[Regeneration support](../../scripts/scientific_reference.py) ·
[DTA-054 evidence and generator footprint](../design_review/test-audit/oracles-and-acceptance.md#dta-054)

**Proposed verification.** In each affected comparator, pair an actual valid witness with a
wrong value whose candidate allowance has been inflated beyond the error. Perturb the sIPOPT
prediction separately from the native observation. Reject missing quantities, nonfinite
inputs, mismatched coordinates, and a true unit or datum mismatch. Preserve a supported
equivalent-unit witness where conversion is part of the comparator. For provenance, change the
358 K case's input or substitute the 368 K attribution and show that the receiving comparison
rejects that mismatch. For regeneration, alter a consumed authored input while keeping the
old golden and show that comparison reports staleness. Preserve the named phase, negative
population, covariance and ordinary original-trial controls.

## <a id="d02"></a>D02 — Admit and observe the actual reference environment

Converge admission for the four legacy exchanges and ordinary reference callers on the existing
locked parity environment and their distinct case requirements. The prepared interpreter and
declared pins enter through the existing caller roles. Bind the invoked isolated Python process
to that interpreter and require its actual package observations to match the selected lock.
Preserve IDAES 2.13.0 and the clean-room reference boundary. Put each required solver capability
with the case or stage that consumes it, including the initialization stage when it governs
the reference. Bind the solver invocation to the admitted prepared executable and capture
the actual executable identity and release in the returned evidence. For direct shell-backed
solves use Pyomo 6.10.1 `SolverFactory` and the selected plugin's `set_executable` and
`executable()` to bind and observe the executable on the instance that runs. IDAES 2.13.0's
heat exchanger `initialize_build` takes a solver name and calls `get_solver`, which creates a
solver separately. The direct binding alone supplies no initializer witness. Determine which
supported selection and observation path reaches that instance and any consumed submodel
initializer. Verify with an admitted executable and a distinct substitute on PATH. The indirect
part of D02 and A04's dependent parity routes require this evidence. Keep a typed nonpass when a
selected stage cannot establish provenance. Keep genuinely analytic stages
independent of solver admission. Share environment checks while leaving model construction,
comparators and each sound wire exchange with their owners.
[Solver configuration overview](../libraries/idaes-pse.md#81-solver-independence-and-solver-configuration) ·
[Pyomo 6.10.1 shell solver](https://github.com/Pyomo/pyomo/blob/6.10.1/pyomo/opt/solver/shellcmd.py) ·
[IDAES 2.13.0 exchanger initializer](https://github.com/IDAES/idaes-pse/blob/2.13.0/idaes/models/unit_models/heat_exchanger.py) ·
[IDAES 2.13.0 solver factory](https://github.com/IDAES/idaes-pse/blob/2.13.0/idaes/core/solvers/get_solver.py) ·
[Parity lock declaration](../../pyproject.toml) ·
[Existing interpreter roles](../../crates/pse-runtime/Cargo.toml) ·
[Host input selection](../../crates/pse-host/src/fixtures/execution.rs) ·
[Exchanger initializer](../../tests/reference/oracles/ordinary_exchanger.py) ·
[PID reference](../../tests/reference/oracles/pid.py)

**Proposed verification.** Exercise the real isolated environment for a declared reference
case, with an actual direct solve and the indirect initializer witness where used. Compare the
returned identity with the selected executable and its prepared provenance. A missing or stale
interpreter, mismatched package, missing solver, or substitution between preflight and use must
produce an attributable nonpass. Preserve required solver capability differences between cases.
Retire per-oracle assertions that infer the performed solve from a second probe. A04 owns the
named parity and reference-unit entry points and their actual admission and selected populations.
[A04](70a-selection-and-execution.md#a04)

## <a id="d03"></a>D03 — Bound exchange output through the performing owner

Use B02's bounded diagnostic child contract for the ordinary and legacy exchanges. Budget the
retained diagnostic prefix and report overflow explicitly while the child runs. On overflow the
process owner continues bounded drain or cancels under the existing finite settlement policy.
Preserve separate facts for child exit, diagnostic overflow, exchange validity and original
settlement. Carry authentic bounded failure diagnostics into the report. The existing stderr-only
Park reader cannot substitute for the finite two-stream contract. Remove unrestricted stdout
and stderr scratch redirection and whole-log error reads in migrated callers.

Apply finite case-derived response limits in both producer and receiver for the legacy JSON
exchanges. Admit the case's result cardinality and variable-length fields before encoding so
the producer's working buffer and file are bounded. Publish the completed response only on
successful encoding. The receiver verifies extent and reads within the limit before
decoding, then checks case identity and semantic completeness. In the ordinary exchange, Python
protobuf 7.36.2 uses a private `DescriptorPool` and `GetMessageClass` to load the selected
binary descriptor, then checks `ByteSize()` before `SerializeToString`. The Rust receiver
keeps the 16 MiB readback cap and uses vendored Buffa 0.9.2 `DecodeOptions::decode_from_slice`
with message, element-memory, recursion and unknown-field limits. These mechanisms bound the
ordinary wire path while preserving the original binary envelope. PSE owns custody before
reading, descriptor and case correspondence, and scientific completeness. Avoid format
consolidation that would lose accepted distinctions or duplicate semantic validation.
[B02](70b-fixtures-and-process-ownership.md#b02) ·
[Original supervisor](../../crates/pse-diagnostics/src/supervised_child.rs) ·
[Python reference encoding and descriptor limits](../../tests/reference/proto_exchange.py) ·
[Rust decoding and named limits](../../crates/pse-runtime/src/reference_exchange.rs) ·
[Buffa overview](../libraries/proto/Buffa.md) · [Exact maintained Buffa delta](../../vendor/buffa/pse-patch.md)

**Proposed verification.** Exercise a child that exceeds the diagnostic limit and returns
zero. The exchange reports overflow, preserves the raw exit and authentic settlement, and
retains only the admitted diagnostic output. Challenge both streams, malformed or truncated
responses, oversized legacy producer output, and a receiver-side oversized payload before
decode. Verify a valid legacy response and ordinary Proto result still reach their existing
comparators. These controls establish the selected channel bounds and failure distinctions.

## Packages

| ID | Status | Prerequisites | Completion |
|---|---|---|---|
| [D01](#d01) | planned | Current independent comparators and authored case identities. E01 closure when the case is moved to selected authored inputs. | Independent comparison allowances reject altered results even with inflated candidate policy. Exact reference attribution and legacy generator stale-input controls exercise their scoped cases. |
| [D02](#d02) | planned | Locked parity environment and named caller roles for direct owner checks. | Prepared interpreter and actual per-stage solver identity are admitted and observed. Substituted or missing tools produce attributable nonpass. A04 consumes this contract for the assembled routes. |
| [D03](#d03) | planned | B02's finite two-stream capture and settlement contract. D02's executed environment identity for assembled reference verification. | Migrated exchanges bound live diagnostics and legacy response production and readback. Zero-exit overflow and malformed response controls pass with authentic settlement, and the Proto bounds stay exercised. |

Use targeted comparator and encoding controls as changes land, then the affected native
exchanges with the repository correctness route and force validation where applicable. At
series scope end, exercise A04's exact selected routes and Plan70's assembled cases through
the performing owners. Keep source correspondence, independent agreement, bounded custody,
and scientific population qualification as separate evidence. Link Plan66l L01/L03 for
DTA-003 and DTA-049. In particular, match `cstr_simultaneous`
(`ad7bac7b5fdd495a990a1bdcd920c35e`) to its exact configured inputs and original result
before crediting executed acceptance. The existence of the seed manifest route establishes
availability. Plan66l owns population and assembled acceptance evidence.
[Simultaneous-controller case](../../packages/reference/campaign/models/cstr-dynamics.pse) ·
[Seed manifest](../../packages/reference/conformance.toml) ·
[Plan66l](66l-required-population-and-assembled-acceptance.md)
