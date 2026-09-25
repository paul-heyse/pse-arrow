---
title: Plan 16 P07 through P09 execution
status: complete
date: 2026-09-25
adrs: [ADR-0088, ADR-0089, ADR-0090, ADR-0093]
phase: 1
evidence: Tested — scoped Linux Rust/native/Python qualification and required static checks
---

# Plan 16 P07 through P09 execution

This packet implements the user-approved P07-P09 plan against the existing P00-P06
foundation in the shared checkout. It preserves concurrent changes.

## Decisions

One contextual eligibility owner serves public preparation and native execution.
Native termination, original numerical qualification, physical usability and available
starts stay separate. Validate coefficient transformation equivalence before transferring
native bounds or optimality claims. Allocation reuse is independent of explicit start
policy; no prior start is the default. Initialization stages transact over immutable
case bindings and retain failed overlays as evidence. Authored tear policies and the
selected revision govern executable strategies; fallback is explicit.

Convert time coordinates once into prepared sample bindings, including nonzero origins.
Reuse Diffsol 0.16.2 event-time and mass-matrix reset sensitivity helpers, retaining
numerical time-partial provenance. Add the user-selected narrow IDAS fixed-mass
ODE/index-1 route for recoverable residual trials and smooth sensitivities. Refuse
unqualified hybrid/recovery combinations. Valve regularization is declared physical
meaning, not a hidden numerical patch. Candidate response derivatives, estimator
qualification, rank and uncertainty are separate.

## Dependency order

1. Shared contracts and ADR-0093 before affected implementation.
2. Contextual routing, derived coefficients and explicit map/cone requests.
3. Native status and original-model qualification, including thread admission.
4. Explicit starts independent of retained allocation, with native state clearing.
5. Transactional overlays and inspectable initialization/recycle strategies.
6. Shared time-coordinate conversion and sample identity.
7. Qualified Diffsol hybrid sensitivities and native IDAS residual dynamics.
8. Declared valve behavior and shared smooth/hybrid fitting qualification.
9. Final integrated and nonfunctional qualification, then evidence-labelled outcome.

## Execution rhythm

During functional implementation use touched compilation and filtered functional units
with force validation. Regeneration is part of a declaration change. Integrated solver,
Python and performance journeys, formatting/lint, governance, generated equivalence,
ADR/documentation and architecture checks wait until all functional scope is implemented.
Delete replaced paths and callers as soon as their targeted replacement tests pass.
Do not introduce compatibility execution paths. P10-P18 remain their own scope.

## Implemented scope

The functional scope and scoped final qualification are complete. The
[implementation self-review](../design_review/reviews/design_review_plan16-native-strategies_2026-09-25.md)
accepts the examined scope. Disk space was restored by the maintainer; no cache cleanup
by the implementation agent was needed. ADR-0093 remains proposed under the decision-PR
process.

The shared selector, derived coefficient projection, original-space qualification,
complete HiGHS upload readback, explicit start policy and transactional stage reports are
implemented. Authored tear cost, policy and grouping govern the selected flow projection.
Public cone, compiled causal-map and revision-bound initialization routes are exposed
through owned Rust/Python strategies. Observation times bind once to prepared samples.

IDAS admits smooth fixed-mass ODE/index-1 systems with recoverable residual trials,
consistent initialization and forward sensitivities. Hybrid IDAS sensitivities,
variable-layout dynamics and general index reduction remain refused. Diffsol's admitted
same-layout hybrid route records its numerical time-partial provenance. Fit response
derivatives remain candidate data; qualified estimates additionally require convergence,
original feasibility and response rank. No covariance or estimator sensitivity is inferred.
New strategy objects expose owned reports; complete publication projection remains P10.
P18 owns the complete performance/solver campaign.

The final process controls expose two additional native limits without weakening policy.
With automatic presolve, the pinned bound-tightening path can return a variable multiplier
that fails complementarity against the original bound. That candidate remains feasible;
the same physical NLP qualifies as stationary with presolve explicitly off. HiGHS' default
QP regularization can miss a stricter requested original objective gap. It remains feasible;
an explicit smaller `qp_regularization_value` qualifies the analytic QP. These are tested
qualification distinctions, not automatic retries or hidden changes to the problem.

## Verification

**Tested — local Linux, baseline zero.** Only targeted functional tests ran during
implementation. Integrated and nonfunctional checks began after the functional barrier.
Rust recipes enable `pse-relations/force-validate`; native runs source the pinned solver
and math environment, execute on the host and use one BLAS/OpenMP thread.

| Command / mode | Evidence and result |
|---|---|
| `just unit-package pse-backend-native 'test(routing::tests::) \| test(unsupported_hybrid_) \| test(native_upload_warnings_) \| test(idas_)' --features pse-backend-native/native-solvers` | Tested: seven passed, zero failed |
| `just unit-package pse-runtime 'test(strategies::) \| test(nonzero_origin_smooth_scheduled_) \| test(initialization_prepares_)' --features pse-runtime/native-solvers` | Tested: five passed, zero failed |
| `just test --features pse-runtime/native-solvers,pse-tests-conformance/native-acceptance --status-level fail --final-status-level fail` | Tested: 1,817 passed, zero failed/skipped; includes conserved back-pressure and original KKT/gap qualification regressions |
| `just test --status-level fail --final-status-level fail`, default workspace | Tested: 1,761 passed, zero failed/skipped |
| `just py-sync-native` | Tested: editable native extension and actual API stubs rebuilt |
| `just py-test -m '"unit or component or integration"'`, fresh native store, Python 3.14.7 | Tested: 141 passed, zero failed |
| `just plan14-python build/plan16-p07-p09` | Tested: four public process journeys passed, zero failed |
| `just quality`, linked native environment | Tested: all Python/repository leaves passed |
| `just adr-lint`; `just docs`; `just architecture-manifest` | Tested: passed; ADRs remain proposed |
| `just check`; `just clippy` (default and no-default workspace/all-targets, `-D warnings`) | Tested: passed after feature-gated import, obsolete lint expectation and report-layout repairs |
| `just doctest`, workspace except the PyO3 cdylib | Tested: all doctests passed, zero failures |
| `just governance`, with the affected `just codegen-relations-check` rerun | Tested: 95 governance tests, all generated comparisons and family checks passed; new generated files registered with intent-to-add |
| `just fmt-rust-check`; `just lint-toml` | Tested: passed |

The original `ci-fast` and governance aggregates retain failed exits from repaired
leaves; they are not represented as successful aggregate invocations. Affected leaves
were rerun to zero, including the complete default test suite. The final native/default
and Python counts above are successful complete runs, not reductions of an accepted
failure baseline. The rebuilt Python boundary was exercised again after report boxing.

Strict extra native-feature FFI documentation/style Clippy retains the existing
ADR-0087 / Plan 14 M22 design-stage exclusion. Ordinary default/no-default Clippy is
required against zero. No lint baseline, dependency pin change, new IDAES numerical
parity, performance, distribution or other-platform qualification is claimed.

## Outcome

### What was built

**Implemented:** one contextual eligibility owner with caller-specific availability;
derived coefficient and explicit cone/map routes; typed native stops and independent
original numerical qualification; complete HiGHS upload equivalence; explicit seeds and
origin receipts independent of native allocation reuse; transactional initialization,
authored tears and compiled causal/Anderson strategies; one prepared time coordinate;
library-owned Diffsol hybrid sensitivity and narrow IDAS recovery/consistent-initialization
paths; declared C2 directional valve physics; and distinct candidate response, rank,
conditioning and estimate qualification. Rust and Python public preparation/results and
generated declarations were migrated together.

The public coefficient-selection boolean, duplicate route admission, implicit
reuse-implies-start behavior, merged initialization specification/result values and
duplicated observation-clock interpretation were removed. No compatibility execution
path was retained. **Tested:** Verification above closes this scoped functional packet.

### A mistake made and corrected

Old native acceptance assertions granted stationarity/optimality from the native return
code alone. The independent checks exposed a presolve multiplier-recovery limitation and
QP regularization gap; tests now verify truthful downgrades and explicit settings that
meet stronger criteria. No tolerance was weakened to make a claim pass. The fixture
generator initially omitted the new valve width and optional time fields; those fields
and the pressure-ratio operation now regenerate from their owner. Feature-gated imports,
the obsolete argument-count lint expectation and the oversized Python result enum were
also repaired in the final phase. An earlier fit fixture retained duplicate oracle
reservations; dropping the completed oracle and sizing its explicit workspace fixed it.

### Deviations from the plan, deliberate

The two integrator routes have separate admitted profiles; this is not a general IDAS
hybrid or higher-index DAE claim. Explicit cone requests use Clarabel's own matrix and
cone types. Starts for fitting remain refused until a fitting-specific seed contract
exists; declared parameter guesses are its current input. Native presolve and QP options
remain explicit choices, with independent qualification and no hidden fallback. Strategy
report publication, general sparse fit/resource work and the full performance campaign
remain P10, P14 and P18 respectively. The existing native strict-lint exclusion and
proposed ADR acceptance boundary above remain in force.
