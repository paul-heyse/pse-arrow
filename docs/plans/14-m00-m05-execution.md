---
title: Plan 14 M00–M05 execution
status: done
date: 2026-09-24
adrs: [ADR-0082, ADR-0083, ADR-0084]
phase: 1
---

# Library-owned math foundation and hard cutover

This is the completed M00–M05 checkpoint. The later
[M10–M14 packet](14-m10-m14-execution.md) supplies native execution, building on
[M06–M08](14-m06-m08-execution.md) and [M09–M10](14-m09-m10-execution.md).
The [foundation contract](14-math-foundation-contract.md) records the present target;
M15 is next. This packet's evidence retains its original scope.


The approved boundary removes the old public math/solve routes. Native solving returns
through later Plan 14 packages. There is no compatibility compiler, evaluator or artifact
reader. This packet supersedes the main plan's later ownership of the deletions needed
for this cut, without implementing those later replacement capabilities.

**Current-state pointer:** this packet's completion receipts are historical. M10–M14
has since implemented the native solver lifecycle; see the
[current packet](14-m10-m14-execution.md) and [inventory](14-execution-inventory.md).
M15 is the next package; full qualification remains M22.

## Ordered work

| Package | Work and unit acceptance |
|---|---|
| M00 | Proposed ADRs; fresh target inventory/manifest; active Plan 14 selection; reject old seals, inherited mappings and premature qualification. |
| M01 | Locked Symbolica/FeOS/num-dual/faer composition; explicit features and native ABI profiles; lazy guarded blocks, fallible multi-output provider, exact local derivatives, clone isolation and alias-Hessian controls. |
| M02 | Formal slots, physical/source contracts, domain/smoothness/provider requirements, class-specific problem profiles; one function capability authority; no registration-only availability. |
| M03 | Physical inference over authored syntax before CAS; typed Symbolica construction; specialization-local bodies and versioned semantic identity; cut all old mathematical callers and delete their engine/transport/tests. |
| M04 | Finite/filtered/ragged domains and reductions; stable semantic binding; aliases and cross-unit connections; separate structure from values; bound preparation and reuse known local shapes. |
| M05 | Pre-normalization obligations; safe dependency order and lazy branches; scalar interpreted library blocks; value versus C1/C2 admission; attributable errors without stale state. |

## Contracts and decisions

- Authoring syntax and quantity rules precede Symbolica. No second executable arithmetic IR.
- Body specification, instance binding, case structure, case values and mutable evaluator are separate.
- Semantic hashes use pse-ids, never Symbolica handles, printed atoms, source spans or instance IDs.
- Symbolica 3.0.0 disables defaults; GMP/MPFR only. Host owns tracing/allocator/thread policy.
- FeOS 0.10.1 uses num-dual 0.14.2; provider types remain private. PC-SAFT is the first data profile.
- Guarded blocks are the production baseline. JIT/SIMD/monolithic conditional execution remain unavailable.
- Providers return typed results outside scalar callbacks. Symbolica owns chain differentiation;
  providers supply their partial derivatives through explicit evaluation-local bindings.
- KINSOL is the future root default; qualify bundled SUNDIALS 7.1.1 through sundials-sys 0.6.2.
- HiGHS 2.4.0 / highs-sys 1.14.3 bundles HiGHS 1.14.0. Clarabel excludes faer-sparse/SDP.
- Diffsol's first target is native ODE/index-1 mass-matrix equations, not general DAE/index reduction.
- Oximo is not added without a concrete consumer. No project Newton, AD tape or sparse algebra engine.

## Cutover and verification

Move the necessary X01–X05, X07–X11 deletions into M03–M05: mathir/numerics,
old callbacks and prepared-case routes, expression relation generators, old Salsa products,
Pyomo/NL stubs and dependent fixtures. Retain source/quantity/containment/port semantics
only for named new consumers. Preserve historical documents, not obsolete production code.

Use targeted units, compilation and required generation. Rust correctness recipes pass
pse-relations/force-validate. Full integration, component, solver, Python, performance and
static/documentation qualification remains M22, with a zero failure baseline.

M05 completion does not establish a working simulator: M06 provider completion, M07
solver derivative artifacts, M08 sparse views, M09 structure, M10 artifact reuse and the
native solver/public runtime packages remain required.

## Outcome

Implemented all six packages and their approved hard-cut deletions. The
[foundation contract](14-math-foundation-contract.md) records the exact supported
scope, library profiles, reuse/deletion decisions and proposed blueprint amendment.
The [scoped review](../design_review/reviews/design_review_math-foundation-cutover_2026-09-24.md)
accepts these boundaries. ADRs remain proposed until their decision PR is accepted.

The old public math/solve routes, custom expression/differentiation/evaluation crates,
MathIR query products, unsupported graph adapters, expression transports and production
Pyomo/NL stubs are gone. A bounded `BodyStore` reuses local compiled evaluators;
worker clones own their scratch. Actual provider factories, physical-before-CAS
construction, finite group membership, canonical binding and lazy guards are enforced.

**Mistake corrected:** eager CAS normalization could fail on an invalid constant in
an untaken branch. Original operands now cross domain barriers before unsafe arithmetic
is constructed. A regression covers the untaken `log(-1)` case. Canonical formal units
are checked explicitly to prevent Celsius values being interpreted as Kelvin.

**Deliberate deviations:** the approved deletion scope moved forward from later
packages. No public solve compatibility path remains. Interpreted values are the
baseline; derivative-neighborhood admission is not a derivative-output implementation.
The no-consumer incidence/DM/containment/tear code was deleted; M09/M14 build the target
library paths. Foreign allocation interception and end-to-end resource closure remain
M10/M21 work.

## Verification

**Tested**, zero failure baseline, Linux Rust 1.98.1. Rust unit recipes use nextest's
default parallel mode and explicitly enable `pse-relations/force-validate`. Symbolica
runs use the locally provisioned multi-core license; key material is excluded from
version control and evidence.

| Command | Passing controls; failed / baseline |
|---|---|
| `.venv/bin/python -m unittest scripts.tests.test_implementation_phase scripts.tests.test_validation` | 34; 0 / 0 |
| `just unit-package pse-math 'package(pse-math)'` | 14; 0 / 0 |
| `just unit-package pse-compiler 'package(pse-compiler)'` | 9; 0 / 0 |
| `just unit-package pse-backend-native 'package(pse-backend-native)'` | 2; 0 / 0 |
| `just unit-package pse-structural 'package(pse-structural)'` | 6; 0 / 0 |
| `just unit-package pse-tests-conformance 'package(pse-tests-conformance)' --features math-composition,native-profiles` | 8; 0 / 0 |
| `just unit-ipopt-abi` | 1; 0 / 0; links/runs in the digest-pinned solver container, no solve performed |

Three additional isolated Rust scope controls pass (0 failures / zero baseline):
`just unit-package pse-tests-governance 'test(target_registry_has_no_legacy_arithmetic_transport_or_placeholder_normalizer) or test(every_crate_directory_is_registered) or test(blueprint_crates_all_exist)' --test registry_governance --test every_crate_registered`.
This checks the exact removed schema/algorithm and crate boundaries, not the governance aggregate.

`just check` compiles the locked workspace and all targets. `just codegen-contracts`
and `just conformance-fixtures` regenerate the changed boundary owners. `just py-sync`
refreshes the locked editable extension with force-validation and regenerates the native
Python stub from compiled PyO3 metadata. Cargo reports
an upstream future-compatibility notice for `proc-macro-error2`; this is not a claim
of full static qualification. Formatting, lint, governance aggregates, installed Python journeys,
component/integration/solver journeys, performance and final evidence/seals remain M22.
