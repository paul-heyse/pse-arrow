---
title: Final cleanup and executable native acceptance
status: complete
date: 2026-09-24
adrs: [ADR-0082, ADR-0083, ADR-0084]
phase: 1
evidence: Implemented and targeted Tested — M19-M20 complete; process acceptance is compiled/discovered but unqualified until M22
---

# M19–M20 execution

Implements the approved M19–M20 slice of [Plan 14](14-library-owned-process-simulator.md).
Author missing acceptance tests and fixtures now; compile/discover them and run isolated
mechanism units. Execute integrated/native process journeys and measurements only in M22.
No Plan 13 campaign, acceptance identity or seal authorizes this target.

## Dependency-ordered work

| Work | Deliverable | State |
|---|---|---|
| M19.1 | Delete unused pse-plans/pse-kernels-ext, compiled spatial families and obsolete expression payload generation | complete |
| M19.2 | One AST partition/serialization pass with syn path traversal | complete |
| M19.3 | Compact bit-exact physical fixtures with production admission | complete |
| M19.4 | strum leaf vocabulary; explicit codec/pool dispositions; dependency closure | complete |
| M20.1 | Version 3 requirement/case/profile manifest and strict inventory status | complete |
| M20.2 | Exact test discovery, linked profiles and phase-based execution guards | complete |
| M20.3 | Complete-witness receipts and bounded current-source provenance | complete |
| M20.4 | Q01–Q18 concrete tests, shared process fixtures and independent oracles | complete |
| M20.5 | Compile acceptance bodies, run isolated controls, update current documents | complete |

## Contracts and library choices

Keep authored finite domains, quantity meanings and selected source/result boundaries.
Remove spatial output contracts without a consumer in the fixed-mass BDF profile.
Keep family pin anchors even where only transitive consumers exist. Normal Python
dependencies retain their actual marshalling roles; independent numerical references
remain test-only. Delete obsolete callers and fixtures together with their mechanism.

Use syn::VisitMut instead of token-string replacement and strum 0.28 for leaf enum
parsing/display/variant enumeration, retaining constant spellings, codes and serde wire
semantics. Compact fixture IDs/dimensions and emit typed enum variants; preserve all
floating-point bits, preconditions and ordinary QuantityRegistryBuilder validation.

Defer serde_arrow: its explicit-field support does not resolve binary-ID serialization,
semantic admission, metadata and allocation contracts. Retain final-buffer leases and
fallible admission: DataFusion's Arrow pool adapter grows reservations infallibly.
Use existing peak/consumer pool instrumentation for tracked memory and separate RSS
measurements for total native allocations. Reconsider replacements only for a concrete
measured surviving consumer, preserving these contracts.

## Acceptance implementation

Extend the existing acceptance manifest rather than creating another evidence platform.
Requirements reference mandatory exact test witnesses, profiles and independent oracles.
Case declarations cannot declare themselves qualified. Q01–Q17 are functional; Q18
requires measurements and separate independent G1–G7 reviews. M22 is a valid case owner.
Nextest JSON and pytest collection establish test identity, not success. Scope listing
does not compile, import the product, execute tests or create receipts.

All missing Qxx test bodies are implementation deliverables. Reuse current unit witnesses
only with a target-specific rationale. Add shared authored heater/flash/separation/recycle,
vessel and steady/transient/mixed fitting fixtures through current public APIs. Cover both
NLP backends, KINSOL, eligible HiGHS/Clarabel classes, refusals, publication faults and
resource/lifetime controls. Use analytic/exhaustive controls, independent frozen teqp
0.23.1 PC-SAFT references and DIPPR100 caloric integrals; matching solver answers are not
independent mathematical validation. Generate references in an isolated CPython 3.12
environment; production tests consume offline data.

Receipts bind complete selected/executed identities, source/fixture/native profile,
artifact schemas and all required witnesses. Reject skipped, missing, stale, duplicate,
interrupted, wrong-profile and historical evidence. One product-input inventory owns
source sealing and continuation; exclude build output, secrets and unrelated library
corpora. Native journeys use the linked extension and explicit force-validation.

Criterion workloads include complete cold/warm operations and teardown, one/eight/32
heater blocks, and one/four-thread profiles. Native nested thread counts remain one.
Each of the 12 scenarios runs in a fresh process; warm runs retain the source/compiler
owner and tear down each completed attempt. Report Criterion samples/confidence
intervals, pool-accounted peak and process-lifetime RSS separately. Bind actual binary,
linked libraries, toolchain and artifact hashes. No performance execution before
current functional qualification.

The [manifest](14-acceptance-cases.toml) contains 29 case groups and 135 exact required
witnesses: 109 Rust tests, three Python tests, four tooling tests, 12 measurements and
seven independent review decisions. Source declarations are shared between Rust and
Python. Current fitting/initialization controls retain weighted loss, implicit
response, singularity, uncertainty, unsupported Hessian and atomic update checks.
Component and integration classifications reflect execution scope even when bodies
live inside Rust library test modules.

The offline thermodynamic fixture includes five homogeneous states and a ternary
two-phase equilibrium. The latter uses teqp chemical potentials and SciPy's bounded
least-squares solve, with independent pressure/equilibrium/material-balance checks.
It establishes a specified equilibrium reference, not global phase stability. FeOS
stability calculation has a separate explicit gas-state witness. Decimal reference
values also check real-algebra values across optimizer and derivative profiles.
The selected evaluators are interpreted; JIT/SIMD remain unqualified.

## Verification

All counts below use a zero-failure baseline; selection/discovery does not execute tests.

| Evidence | Command and conditions | Result |
|---|---|---|
| Tested | `just unit-package pse-codegen 'test(rust::semantic::tests::) or test(foundation_unit_semantic_values)'`; explicit Arrow force-validation | 2 passed, 0 failed |
| Tested | `just unit-package pse-quantity 'test(enums::tests::)'`; explicit Arrow force-validation | 4 passed, 0 failed |
| Tested | `just unit-physical-fixture`; explicit Arrow force-validation, bit-exact source fixture through registry admission | 1 passed, 0 failed |
| Tested | `just unit-package pse-math 'test(guarded_tests::high_precision_real_values_survive_optimizer_and_jet_profiles)'`; force-validation and configured Symbolica license | 1 passed, 0 failed |
| Tested | `just unit-plan14-sources`; linked native Python extension, generated codecs only | 1 passed, 0 failed |
| Tested | `.venv/bin/python -m unittest scripts.tests.test_implementation_phase scripts.tests.test_validation scripts.tests.test_plan14_acceptance`; isolated tool mode | 38 passed, 0 failed; the four `just plan14-tools` controls are a subset |
| Interface-checked | `just plan14-discover`; full native features, explicit force-validation, host linked libraries | All 109 Rust and 3 Python identities found; no test execution |
| Interface-checked | `just check`, `just check-native-python`, and native conformance/benchmark all-target compilation | Passed; compiled acceptance bodies and native boundary |
| Implemented | Registry/codegen regeneration and `just conformance-fixtures` | Produced current contracts/fixtures; full regeneration comparison deferred |

Dependency analysis via `just audit-machete` was advisory. Confirmed unused direct
dependencies and empty crate registrations were removed; macro consumers, shared
fixture imports and family pin anchors remain. The final default backend compilation
also passes after restricting its unused native-only test helper to its feature
consumers. The upstream `proc-macro-error2` future-incompatibility notice remains an
M22 qualification concern. No clean full lint/governance campaign is claimed.

Full quality, generation comparison, native process execution, installed Python and
publication journeys, performance and formal design qualification remain **not_run**
for this slice. Discovery artifacts precede the final documentation update and are
compile evidence, not a final source seal or qualified receipt.

## Outcome

**Implemented:** M19 removes both empty crates, unused compiled spatial families and
obsolete expression payload generation. One syn AST pass owns emitted semantic
partitioning; strum supplies enum parsing/display/variant iteration without changing
wire spellings or codes. Physical fixture output shrinks from 1,315,247 bytes/29,688
lines to 406,913 bytes/9,779 lines (69.1% fewer bytes), retaining exact float bits and
production admission. This is source volume, not a compile-time or runtime speedup.

**Implemented:** M20 supplies the missing physical process, coefficient/conic, native
tear, vessel/event, fitting, publication fault, cancellation and Python test bodies,
independent offline oracles, complete workload measurements and review collection.
Version 3 receipts and exact profile witnesses replace prefix/placeholding evidence;
the phase guard prevents integrated qualification before M21 and measurements before
current functional qualification. Independent reviews are inputs, never generated
approvals; their original files and cited evidence are retained and hash-checked.

**Mistakes corrected:** exact discovery exposed stale Rust module paths; those now
use compiled identities. Generated source decoding exposed a constraint-shaped
objective contribution; the fixture now uses the actual objective contract. Dependency
analysis missed imports in shared fixture modules; those dependencies were retained.
The initial thermo probe selected a negative-pressure dense state; the admitted
positive-pressure reference replaced it without relaxing finite-value checks.

**Deliberate deviations:** retain explicit codecs and fallible allocation/final-buffer
ownership because the suggested library alternatives do not meet the surviving
contracts. Ternary flash uses teqp plus SciPy in an isolated reference environment;
neither is a production numerical route. Python exercises the shared heater/recycle
frontdoor; vessel/cone/graph construction uses the public typed Rust services. Existing
useful contract tests enter only with an explicit target/oracle rationale.

**Next:** M21 reviews all final consumers, advertised profiles, D01 retained mechanisms
and deletion closure, gathers current classified development evidence and issues a
fresh implementation seal. M22 then executes Q01–Q17 and whole-repository gates,
repairs failures against zero, measures Q18 and obtains independent G1–G7 decisions.
ADR-0082–0084 remain proposed; formal reconciliation and whole-plan completion stay open.
