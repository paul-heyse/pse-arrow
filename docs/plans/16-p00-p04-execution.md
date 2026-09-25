---
title: Plan 16 P00 through P04 execution
status: done
date: 2026-09-25
adrs: [ADR-0088, ADR-0089, ADR-0090, ADR-0091, ADR-0092]
phase: 1
evidence: Tested — P00–P04 targeted tests, force-validated Rust regressions and linked Python journeys; scoped results below
---

# Plan 16 P00 through P04 execution

This packet executes [Plan 16](16-data-model-architecture.md) P00–P04. The initial
checkout was clean at `239a8b55`; existing main and Cargo paths remain in use.

## Scope decisions

The existing personal Symbolica license/configuration is used. No license-management
feature, restricted mode or process-isolation framework is added. Symbol registration
moves to explicit initialization, with semantic identity and numerical agreement as
the reproducibility promise, not blanket bitwise identity.

FeOS starts with homogeneous explicit-density execution. Phase-equilibrium requests
are refused; flash derivatives and execution remain outside this packet. Parameter
selection uses checked records, not `subset`, whose pinned implementation can panic
and discard association details. Fixed-size dual seed blocks support arbitrary admitted
composition width while satisfying FeOS's `D: Copy` contract. PC-SAFT/DIPPR own physics.

Integration, broad repository checks, formatting/lint and performance runs wait until
the full requested functional scope is implemented. During implementation use only
targeted compilation/functional tests and required code generation. Full Plan 16
qualification is P18; no P00–P04 result closes the whole plan.

## Dependency-ordered execution and coverage

| Packet | Existing owner and enforcement | Work and deletion | Targeted evidence | Review coverage |
|---|---|---|---|---|
| P00 | ADRs 0088–0092; blueprint current contract | Resolve target ownership and authority routes; preserve ADR-0087/M22 history | Design/interface inspection, no runtime claim | N01–N10, F27 |
| P01 | pse-math typed/library boundary; compiler typed admission | Exact signed integer/rational powers, retained guards; explicit initialized symbol context and effective environment; delete float-versus-rational structural check and hidden registration | Cubic, square, reciprocal and rational analytic jets; guard failures and registration ordering | F03/F04 |
| P02 | Registry/pse-model vocabulary; pse-ids framing; diagnostics | Shared tags including heat/time/evidence; scoped identity projections, signed-zero equality, semantic schema digest; structured errors/capability foundations | Codec round trips, mutations/invalidation, unknown tags, source diagnostics | F06/F10–F12/F16/F18/F24; N03/N05–N07 |
| P03 | workflow selected admission and compiler Inputs | Shared builder/document boundary; template/instance binding; exhaustive selected support; real flows; atomic revision preparation; declaration-producing vessel/heater; delete fallthrough/empty flow/bypass paths | Reordering, unrelated cases, unsupported selected rows, finite/ragged domains and guards, fanout, interleaving revisions | F01/F02/F24; N01/N04 |
| P04 | material/quantity and kernel registration | Explicit species/record mapping and provenance; variable-width derivatives; complete port typing and H/S references; declared stability; checked molar reaction/heat contributions | Pure/binary/ternary/larger data binding, permutation, analytic/differential jets, entropy reference, kind mismatch and element closure | F02/F08/F10; N02 |

## Semantic scopes

Body content excludes diagnostic IDs/spans/docs. Revision includes instance binding
and topology. Case includes roles, values, bounds, parameters and guesses. Analysis
includes requested mode/outputs/derivatives/policy. Prepared artifacts additionally
include consumed constants, build/profile and effective library environment. Layout
includes ordered coordinates and sparsity, excluding coefficient-only changes. Used
starts, results and publication have separate downstream projections. Floating frames
and equality preserve signed zero. New semantic schema identity is additive until the
P12 durable compatibility migration.

## Physical contract details

Species identifiers map explicitly to selected PC-SAFT and DIPPR identifiers. Reject
duplicate/missing records and conflicting binary data before `Parameters::from_records`;
missing interactions require explicit zero policy. Use T, molar density and N−1 ordered
fractions with an explicit dependent species; reject noninterior multicomponent trial
states without clipping or renormalization. One FeOS State per seed block serves all
requested outputs. Jacobians/Hessians are row-major and bounded.

Enthalpy and entropy references are separate. Bundled enthalpy uses ideal-gas zero at
298.15 K without formation. Entropy uses pure ideal-gas zero at 298.15 K and 1 bar,
including mixing/residual terms. The raw FeOS entropy datum is derived from its
reference units: P*=kB·1 K/(1 angstrom)^3; conversion subtracts R·ln(P*/1 bar).
An independent reference must establish this before adopting the correction. Reference
corrections stay inside dual arithmetic. No absolute third-law entropy is claimed.

Mechanical stability, global phase stability, failed checks and unknown checks are
distinct. Reactions retain diagnostic-friendly authored storage; selected execution
requires molar homogeneous stoichiometry, element closure and explicit energy data or
heat. Equilibrium/multiphase reactions are refused. A methane + propane → 2 ethane
fixture is a constructed conservation example, not an empirical kinetic claim.

## Supported boundary

Template lowering supports explicit finite scalar/indexed variables, parameters,
equations, guards, scalar conservation laws, child instances and typed connections.
The existing compiler's explicit finite/ragged group bindings remain supported.
Template parent-domain expansion, inherited features, continuous domains, derivative
symbols, implicit property/reaction packages, selected scaling and tear strategies
are source-attributed refusals. Their storage is not advertised as execution.
Material bindings use explicit native-provider material-system IDs and generated
reaction applications; they do not reinterpret legacy package declarations.

The vessel convenience entry now produces inspectable ordinary computation, balance
and dynamic declarations and feeds the same freeze boundary. Its fixed-composition
recipe is an example, distinct from the variable-width homogeneous FeOS provider.
The reaction projection shares the authored rate output across balances using
positive stoichiometric multipliers; it creates no second rate evaluator. Explicit
heat is required. Equilibrium, multiphase and implicit concentration models are refused.

## Execution status

P00 contract documents and P01–P04 functional changes are implemented. Targeted tests
cover exact powers and jets, symbol-registration ordering in fresh processes,
semantic identity, shared Arrow/JSON tags, selected templates and connections,
multi-case laws, independent homogeneous entropy, variable-width derivatives and
element/material/energy closure. The public Python diagnostics expose class, stage,
rule, source IDs and typed observations.

Deferred scoped integration and repository checks have completed after the complete
requested functional scope. ADR acceptance remains on the normal decision PR route.
P05–P18, full-plan acceptance, performance claims and independent final review remain
outside this packet. The current Plan 14 campaign recipes are used as regression
checks, not as Plan 16 acceptance authority.

## Verification

All Rust functional commands below explicitly enable `pse-relations/force-validate`;
the required failure baseline is zero. Targeted tests ran during implementation.
The broader runs began only after the requested functional scope was implemented.

| Evidence | Command / exercised scope | Result |
|---|---|---|
| Tested | `just unit-package pse-compiler` with authored-power, normalized-cubic, signed-zero and definition-rename filters; separate finite/ragged filter | 4 + 1 passed |
| Tested | `just unit-package pse-math` with symbolic-context and fresh-process admission-order filters | Concurrent registration and reversed process order passed |
| Tested | `just unit-package pse-schema 'test(semantic_contract_separates)'` | Semantic versus descriptive/transport changes passed |
| Tested | `just unit-package pse-runtime` with composition, reaction, contribution, vocabulary, document-equivalence and selected-flow filters | 13 passed; additional integer-parameter and vessel-coordinate controls passed |
| Tested | `just unit-package pse-kernels` with FeOS reference, permutation, derivative, kind and stability filters | Independent pure/binary/ternary entropy, wide derivative blocks, H/U refusal and mechanical-policy controls passed |
| Tested | `just test`, then `just test-package pse-tests-conformance --test invariant_fixtures` after regeneration | Initial workspace run: 1,714 passed, 7 stale-fixture failures; affected target: 733 passed, 0 failed |
| Tested | `just plan14-native`, current native feature graph, one native thread | 127 passed, 0 failed; regression scope, not Plan 16 acceptance |
| Tested | `just doctest`, workspace excluding the PyO3 cdylib | Passed, 0 failures |
| Tested | `just family-check` | Pinned families, evidence locks and dependency ceilings passed |
| Tested | `just py-native-contracts`, rebuilt linked native extension | 28 passed, 0 failed |
| Tested | `just plan14-python build/plan16-p00-p04-python-final`, rebuilt linked native extension | 4 public workflow regressions passed, 0 failed |
| Tested | `just py-test`, fresh native store, Python 3.14.7 unit/component mode | 135 passed, 0 failed |
| Tested | `just governance` | 95 governance tests, every codegen comparison and family check passed |
| Tested | `just clippy-no-default`; `just clippy-default` after the constructor annotation repair | Both feature modes passed with `-D warnings` |
| Tested | `just quality`, then `just lint-imports` after editable installation completed | All quality leaves passed; the initial import check overlapped the temporary package uninstall |
| Tested | `just fmt-rust-check`, `just docs`, `just architecture-manifest` | Passed |
| Tested | `just adr-lint`, then `just adr-frontmatter-check` after correcting evidence fields to exact labels | All ADR leaves passed; records remain proposed |

Affected failed checks were repaired and rerun; the entire workspace test suite was
not repeated after fixture regeneration. Cargo reports an upstream future-compatibility
notice for `proc-macro-error2 2.0.1`; mdBook reports a large search index. Neither is
represented as a compiler/lint failure or as resolved by this packet.
No performance, empirical property certification, distribution or other-platform
claim follows from these checks.

## Outcome

**Implemented:** selected reusable composition, exact authored powers, explicit symbolic
initialization, registry-owned result/capability tags, semantic projections and structured
errors. FeOS consumes checked records and arbitrary admitted component counts, with
library dual-number blocks, separate H/S references and declared stability. Conserved
reaction contributions reuse one authored rate and explicit heat. The vessel now emits
ordinary declarations and maps actual component order. Shared function vocabulary moved
to the quantity-contract layer, eliminating a generator-to-generated-model dependency.

**Tested:** targeted packet controls and the scoped regression/check commands above
meet the zero-failure baseline after affected repairs. Integrated and nonfunctional
checks were deferred until all P00–P04 functional work was implemented. Full Plan 16
qualification remains P18.

**Mistakes corrected:** conservation-law IDs initially lacked a case namespace;
reaction admission policy was initially absent from revision identity; a fixed vessel
coordinate order could misbind a permuted dataset. Their targeted controls now pass.
Schema generation did not also refresh invariant fixtures; the separate generator and
affected conformance run corrected the seven failures. Host codegen needs the linked
solver search path with Cargo's container runner unset; Python checks must follow the
editable installation rather than overlap its temporary uninstall step.

**Deliberate boundaries:** the supported/refused formulations above, personal-license
scope, proposed ADR acceptance route, and P05–P18 deferral remain explicit. The added
semantic schema fingerprint does not replace the durable compatibility check before
P12. Static capability inventory is not contextual eligibility; P07 owns selection.
