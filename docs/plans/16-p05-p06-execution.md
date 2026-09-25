---
title: Plan 16 P05 and P06 execution
status: done
date: 2026-09-25
adrs: [ADR-0088, ADR-0089, ADR-0090]
phase: 1
evidence: Tested — P05–P06 functional scope, force-validated Rust suites and linked Python journeys; static exclusions below
---

# Plan 16 P05 and P06 execution

This packet implements [Plan 16](16-data-model-architecture.md) P05–P06 in the existing
checkout, preserving the P00–P04 work. The approved conversation plan supplies the
execution contract below. No dependency upgrade or new crate is planned.

## Decisions

Resolve numerical meaning once, with precedence analysis override, selected case
priority, explicit model target, explicitly bound property default, quantity nominal,
then a recorded canonical-unit nominal of one. Equal-priority conflicts fail. Absolute
and relative tolerance magnitudes have explicit coordinates; relative budgets use
frozen nominals, never trial values. Strict completeness is optional.

Normalize before presolve and transport derivatives, sparse coefficients, starts and
duals reversibly. Native algorithmic scaling is separate. Integer coordinates preserve
their lattice; nonpolyhedral cone rows share one positive scale. Hard guards are never
relaxed. Confirm candidate presolve infeasibility on per-bound tolerance-expanded
original normalized facts using library propagation, without auxiliary elimination.

Assess original numerical feasibility and physical conservation once at completion.
Failed closure retains the candidate and makes it unusable by default. Explicit
analysis opt-in permits qualified use without claiming closure; an unavailable required
check remains unusable. Integration accuracy, integrated observables, instantaneous
balances and cumulative closure remain separate requirements.

Share admitted derivative/domain and bound-dependent mathematical facts. Symbolica
owns algebra; pounce-presolve owns interval propagation; graph libraries own matching,
ordering and cycle witnesses. Stable semantic block identity is separate from position.
Exact Gram evidence remains the default. Numerical PSD qualification requires explicit
opt-in and tolerances, resource-bounded faer eigenanalysis and residual qualification.
Never repair or replace the original quadratic matrix.

## Dependency order

1. Extend proposed contracts and registry-owned numerical requests/resolved vocabulary.
2. Establish admitted derivative capabilities and shared bound/coefficient facts.
3. Implement reversible normalization and source-coordinate transport.
4. Correct FBBT projection and presolve qualification/certification.
5. Complete structural identity, witnesses and convexity evidence.
6. Wire existing native adapters and workflow assessments, migrate public callers and
   delete replaced mechanisms.
7. Qualify the complete scope once: integration and public journeys, then formatting,
   lint, governance, generated equivalence, documentation and architecture checks.

## Execution rhythm and boundary

During steps 1–6 use touched compilation and explicitly filtered functional units with
force-validation. Run code generation as part of schema changes. Integration, solver
journeys, formatting and other nonfunctional checks wait until all functional scope is
implemented. The startup report's stale Python environment and missing native library
must be repaired through supported bootstrap/sync recipes before Python validation.

P07 routing/status expansion, P08 strategy policy, P09 dynamics redesign, P10–P12 broader
result/cache/persistence migrations and P18 full-plan/performance qualification are not
closed by this packet. Existing Plan 14 campaigns supply regression evidence only.

## Current state

The registry owns numerical requirements, explicit provider-output/default bindings,
resolved numerical provenance and candidate assessments. The model/math resolver handles
precedence, magnitude-unit conversion, frozen budgets, integer coordinates and recorded
fallbacks. Contract generation has run. Shared mathematical facts distinguish available
and prepared derivatives, affine rows, objective degree, bound assumptions and original
domain obligations. Symbolica's built-in constants and normalized transcendental forms
are admitted without inventing model parameters. Coefficient extraction consumes these
facts; pounce-presolve owns propagation and retained infeasibility evidence.

The shared NLP pipeline transports model coordinates, derivatives, starts and duals;
coefficient, root and cone adapters have corresponding checked transport. Independent
native feasibility/KKT/gap controls derive from the resolved policy. Original-bound
relaxation is disabled; KINSOL receives strict sign guards. Cone-block adjustments are
retained in resolved provenance. Numerical PSD evidence is explicit, bounded and tied
to its policy, coordinates, coefficient assumptions and unchanged matrix. Structural
products preserve block identity/order, conditional partition scope and cycle edges.

Public solve, fitting, simulation and conditional initialization preparation consume the
numerical policy. Integrated-flux accuracy and cumulative closure retain separate targets.
Completion stores physical assessments once, separately from native termination and
numerical feasibility. Candidate usability and the selected/overridden numerical sources
are exposed through generated result tables and the Python boundary. Native infeasibility
proofs retain original row/column/contribution identities and per-bound budgets.

Targeted checks have exercised resolver precedence/units, exact/numerical convexity,
admitted transcendental projection, structural identity/cycles, native stopping options,
strict signs, presolve certificates and coordinate recovery. All functional scope is
implemented and the final qualification stage is complete within the boundary below. The public result
encoding control distinguishes numerical feasibility from failed physical closure and
explicit qualified use. The final workspace, native, generated and public Python
checks passed after the affected repairs recorded below.

Existing sequence-continuation strategy remains owned by P08; this packet qualifies final
candidate use. P07 retains broader solver-class/status qualification and public routing
expansion. P09 retains redesign of integration/event strategies; local integrator error
controls remain distinct from frozen output/closure acceptance.


## Qualification boundary

The ordinary workspace default/no-default Clippy gates remain required. The additional
native-feature strict Clippy diagnostic encounters the pre-existing FFI documentation
and style work explicitly excluded by ADR-0087 and the Plan 14 M22 design-stage static
boundary. This packet does not claim that surface clean or alter lint configuration.
Findings introduced by this packet are repaired. The zero baseline remains the required
state for the gates this packet claims, and ordinary merge/release requirements remain.

The [implementation self-review](../design_review/reviews/design_review_plan16-numerics-facts_2026-09-25.md)
records core/profile gates, boundary contracts and library alternatives. Proposed
ADR-0088/0089/0090 retain their existing decision-PR acceptance route.


## Verification

**Tested — local Linux, baseline zero.** Targeted functional tests were the only tests
run during implementation. All integrated and nonfunctional commands below began after
functional completion. Rust test recipes explicitly enabled
`pse-relations/force-validate`; linked native runs used the pinned solver/math environment,
host execution and one BLAS/OpenMP thread.

| Command / mode | Evidence and result |
|---|---|
| `just unit-package pse-math` with numerical-policy, convexity, mathematical-facts and admitted-transcendental filters | Tested: six passed, zero failed |
| `just unit-package pse-structural` with structural-identity, conditional-schedule and forbidden-cycle filters | Tested: three passed, zero failed |
| `just unit-package pse-backend-native` with numerical-options, strict-guards, coordinate-transport, presolve-certificate and normalization-callback filters; native-solvers | Tested: six passed; separate root and coefficient/certificate recovery filters each passed |
| `just unit-package pse-runtime` with property-default-binding, candidate-assessment, compiled-weighted-loss, all-fixed-required-presolve and solver-profile-refusal filters; native-solvers | Tested: five passed, zero failed |
| `just test`, default workspace via `just ci-fast` | Tested: 1,752 passed, zero failed/skipped |
| `just test --features pse-runtime/native-solvers,pse-tests-conformance/native-acceptance` | Tested: 1,797 passed, zero failed/skipped; includes public numerical/closure encoding regression |
| `just py-sync-native` | Tested: editable native extension rebuilt and actual compiled API stubs generated |
| `just quality` | Tested: all Python and repository-quality leaves passed |
| `just adr-lint` | Tested: front matter, generated index and register checks passed; ADRs remain proposed |
| `just docs`; `just architecture-manifest` | Tested: book build and manifest checks passed |
| `just doctest`, workspace except the PyO3 cdylib | Tested: all doctests passed, zero failures |
| `just clippy-default`; `just clippy-no-default`, after the slice-reference repair | Tested: both passed with `-D warnings` |
| `just governance`, with `just codegen-relations-check` rerun after conflicting generator invocations completed | Tested: 95 governance tests, all generated comparisons and family checks passed |
| `just py-test -m '"unit or component or integration"'`, fresh native store, Python 3.14.7 | Tested: 139 passed, zero failed |
| `just plan14-python build/plan16-p05-p06`, rebuilt native extension | Tested: four public process journeys passed, zero failed |
| `just fmt`; `just fmt-rust-check`; `just lint-toml` | Tested: formatting applied; format checks passed |

The original `ci-fast` and governance aggregates retained failed exits from repaired
leaves; affected leaves were rerun, not represented as successful original aggregates.
The complete default and native test runs above are zero-failure terminal observations.
The extra strict native-feature lint attempt reported 140 findings in the backend test
build against the zero baseline; it is not a clean gate. Its pre-existing FFI documentation
and style scope remains excluded as described above. This is separate from the required
ordinary default/no-default workspace Clippy modes.


## Outcome

**Implemented:** model-owned numerical resolution with selected and overridden provenance,
unit-aware frozen acceptance budgets, checked native coordinate/dual/diagnostic transport,
per-bound presolve certification, original numerical and physical assessments, immutable
candidate usability, shared derivative/domain/affine facts, stable structural block and
cycle identities, and exact or explicitly qualified numerical convexity evidence.
Solve, fitting, dynamics and conditional initialization consume these owners. Public Rust,
Python and generated result contracts were migrated together. The old positional public
tolerance/scaling interpretation and competing affine classifier were removed.

**Tested:** the commands and feature modes in Verification meet the zero-failure baseline
for this packet's claimed surface. The layered implementation self-review accepts the
P05–P06 scope, with the existing native strict-lint boundary stated explicitly. This is
neither whole-Plan-16 acceptance nor a new release/platform qualification.

### A mistake made and corrected

Symbolica's normalized exponential contains its builtin constant E; treating every
reported symbol as a model input rejected valid log/exp projections. The implementation
now uses Symbolica's `is_constant`, and an admitted-transcendental regression exercises
that representation. Final compilation also found a remaining positional fitting-test
caller, which now uses a semantic target requirement. Generated invariant fixtures were
regenerated from the registry.

The first complete native run had four container Cargo-cache failures and one stale
fixture-inventory failure. Host execution and regeneration corrected those causes; the
complete 1,797-test rerun passed. Disk exhaustion interrupted later gates; a previewed,
package-scoped Cargo cleanup removed only regenerable `pse-runtime` artifacts. Concurrent
`cargo run` variants exposed a shared `xtask` executable race; the affected generation and
Python commands were rerun sequentially. No source or other checkout was cleaned.

### Deliberate deviations and remaining scope

Library-default algorithmic scaling remains separate from authored normalization.
Unsupported native scale/bound combinations fail explicitly. Exact Gram proof is still
the default; numerical PSD is opt-in, resource-bounded and never repairs Q. Conditional
structural partitions make no independent-subproblem claim. Existing continuation policy
remains P08; final candidate-use policy is implemented here.

The strict native-feature Clippy diagnostic is excluded under the existing ADR-0087
boundary, not declared clean. Cargo also reports an upstream future-compatibility notice
for `proc-macro-error2 2.0.1`, and mdBook reports a large search index. Neither notice is
claimed repaired. Default/no-default workspace Clippy passes with warnings denied.
P07–P18, performance campaigns, distributions and other platforms remain outside this
packet. Proposed ADR acceptance still follows the repository decision-PR route.
