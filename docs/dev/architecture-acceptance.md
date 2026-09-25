---
title: Plan 14 native simulator acceptance
status: implemented
date: 2026-09-24
adrs: [ADR-0082, ADR-0083, ADR-0084]
---

# Plan 14 native simulator acceptance

**Implemented:** M19–M21 supplies concrete native process tests, independent
references, operating envelopes, contribution-based conservation and measurement
bodies. The [M22 packet](../plans/14-m22-execution.md) records local Linux execution
and the design-stage completion boundary. The
[manifest](../plans/14-acceptance-cases.toml) maps Q01–Q18 to exact witnesses and
profiles. Execution receipts record what was actually tested. No historical acceptance identity, old seal or passing campaign
satisfies a current requirement.

## Current commands

| Command | Purpose and boundary |
|---|---|
| `just assessment-list --phase functional` | Read the current functional scope; no product import, compilation or execution |
| `just assessment-list --phase performance` | Read measurement/review scope; no execution |
| `just plan14-fixtures` | Generate shared authored declarations from the explicit physical recipe |
| `just plan14-reference` | Generate independent offline teqp/Decimal references in an isolated CPython 3.12 environment |
| `just py-sync-native` | Install the native editable Python profile and generate its actual API stubs |
| `just plan14-discover` | Compile/enumerate the exact native Rust tests and collect Python tests without running them |
| `just unit-plan14-sources` | Decode shared sources through generated Python contracts without constructing the runtime |
| `just plan14-tools <directory>` | Run isolated evidence/deletion controls and emit their report |
| `just architecture-manifest` | Check current manifest ownership/source routes |
| `just plan14-development <new-directory>` | M21: run only exact manifest unit cases through Nextest, pytest and unittest; authenticate reports and native identity |
| `just architecture-acceptance <new-directory> --phase functional` | M22: Q01–Q17 and applicable full repository gates |
| `just architecture-acceptance <new-directory> --phase performance --functional-from <functional-directory>` | M22: complete measurements and independent G1–G8 and PS-G1–PS-G3 decisions after functional success |

Assessment and architecture acceptance run directly against current sources. The
M21 source seal was removed at the maintainer's request. Performance campaigns still
require current successful functional evidence. Native test bodies inside `#[test]`
modules remain component/integration journeys when that is what they execute.

## Native and physical profiles

Rust acceptance links the full native solver profile plus conformance acceptance,
with `pse-relations/force-validate`. Python acceptance uses the extension built by
`py-sync-native` with `force-validate,native-solvers`. Recipes source the existing
solver/math environment, set the actual Ipopt/MUMPS runtime path and set OMP,
OpenBLAS and MKL nested thread counts to one. Local Symbolica credentials are loaded
quietly when configured; they never enter source/evidence artifacts. Native linkage,
ABI/profile and license prerequisites must be satisfied; absent capabilities fail.

Shared files under `tests/fixtures/plan14` declare indexed heater/recycle,
optimization, ternary flash and separation cases, providers and physical vocabulary.
Rust adds a conserved vessel with heat input, valve/event/reset and steady/transient/
mixed fitting through public APIs. Python uses the same source declarations for its
native heater/recycle frontdoor. FeOS numerical values are compared against offline
teqp PC-SAFT plus Decimal DIPPR100 caloric references; flash equilibrium is checked
against independently solved pressure, chemical-potential and material balances.
A homogeneous stability check has its own explicit FeOS witness; the flash reference
is not a global stability certificate. Analytic coefficients/cones, exhaustive tiny
graph decisions, exact derivatives and singular/refusal controls supply other oracles.

Publication tests exercise exact reopening, cancellation during actual writes,
retained buffers and source identity. Native limits, candidate availability and
original physical quality remain separate. The compiler currently uses interpreted
Symbolica/Numerica evaluators; JIT/SIMD, general implicit DAE, hybrid sensitivities,
global MINLP and covariance/global-identifiability claims are outside this profile.

## Measurements and independent review input

`plan14-measure` runs Criterion in 23 fresh processes: cold/warm heater operations
with one/eight/32 blocks and one/four process threads; structural edits and new
specializations; flash cold/warm/difficult starts; conserved vessel, transient fitting,
publication/reopening and after-entry cancellation. Complete operations include
case preparation/edit and compilation, joined native solve, physical quality, Arrow result
access and attempt teardown. Cold operations also construct/drop their runtime;
warm operations retain the revision/compiler owner. Every report separates tracked
pool peaks from process-lifetime RSS, and binds raw samples, confidence intervals,
workload shape, binary/linked libraries and toolchain. No speedup is implied by the
presence of a benchmark. Rust application compilation is untimed setup using the
existing target/cache and `dev` profile. Ten flat samples use a 250 ms warmup and
a one-second measurement target, extended automatically for slow operations.
These observations are design-stage costs, not release-profile performance.

Use `--stop-after plan14-measure` to retain measurements with reviews pending.
Resume into a fresh directory using `--resume-from`, `--change-reason` and affected
`--rerun` gates. Documentation-only continuation retains the original measurement
identity; executable changes require new measurements and all source changes
require current independent reviews. Functional observations affected only by reviewed
formatting/style edits may be retained with an explicit change reason and affected
rerun selection, including edits overlapping the original run. Their original drift
and origin remain visible; an already authenticated continuation needs no repeated
impact decision merely to consume it.

Independent reviewers supply one file for every `review_gates` entry in the manifest:
`G1.json`–`G8.json` and `PS-G1.json`–`PS-G3.json`, under
`build/plan14/independent-reviews` or `PSE_PLAN14_REVIEW_INPUT`. Each record requires:

- `gate`, the current bounded `source_digest`, and `verdict` (`Accept` or `Accept-scoped`);
- `reviewer`, a nonempty `implementation_authors` list excluding that reviewer;
- a concrete `scope`, repository-relative `evidence` paths and `open_must_findings: 0`.

The current digest is the SHA-256 of JSON with sorted keys over
`scripts.validation.sources(root)`, matching the measurement collector. Complete
the human review against the final source and qualification evidence before
collecting it. The collector retains and hashes original review files and cited
evidence. Missing decisions, stale sources and unresolved mandatory findings fail;
there is no generated approval or self-review substitute.

The [validation guide](validation-assessment.md) defines receipt version 3, exact
case coverage, classified advisory results and source-aware continuation. M22 must
repair actual functional failures and rerun affected gates. The design-stage scope
keeps default/native tests, Python journeys and repository contract checks. Release,
coverage, exhaustive feature matrices, strict Clippy backlog cleanup and
distribution/platform acceptance are separate. The full workspace test selection
already includes the governance test crate. The known upstream `proc-macro-error2` warning is maintainer-accepted
(ADR-0087); no functional failure is waived.

The development receipt is the existing version-3 `checks.json` plus retained raw
reports and native provenance, with mode `development`. Its pointer lives at
`build/plan14/development-checks.json`. Handwritten pass counts cannot replace actual
execution evidence. Collectors reparse the reports and revalidate actual binaries and
linked libraries, including retained continuation evidence. Changed source or native
bytes require fresh affected evidence. These artifacts are local build
outputs; they are not a committed declaration of scientific acceptance.
