---
title: Consumed runtime settings
date: 2026-10-14
tier: change
purpose: target
status: review
standard: Core 3.4
heuristics: Efficient Architecture 1.0
profile: Process Simulator 1.5
baseline: a6c616166948a79231bbd91546d358bf75955c72
---

# Consumed runtime settings

**Decision: Accept the bounded target design.** Remove
`ResourceBudget::hashing_may_use_pool`, its native construction/projection field,
Python constructor keyword/getter, generated stub entries and obsolete mechanism
tests/literals. Preserve equality of all remaining consumed resource and execution
settings. Do not introduce a compatibility shim or replacement isolation flag.

The setting has no hashing execution consumer in the inspected workspace.
Nevertheless, deleting it intentionally changes observable runtime compatibility:
the Python process-runtime owner currently compares complete `ResourceBudget`
values, so settings differing only in this field cause the second acquisition to
fail. After removal, otherwise identical settings may acquire the same existing
process runtime. The baseline does not create isolated runtimes for differing
values. This is an explicit contract change, not a claim of unchanged behavior.

**Architectural fitness:** satisfied for the proposed change.
**Behavioral/semantic adequacy:** adequate at the proposed-design level.
Acceptance does not establish landed implementation, installed-extension behavior,
scientific qualification or measured performance.

## 1. Scope, drivers and coverage

This independent CHANGE / TARGET review applies Core 3.4, Efficient Architecture
1.0, Process Simulator 1.5 and the pse-arrow binding. The reviewer is the delegated
design reviewer; the disposition owner is
[Plan38f](../../plans/38f-boundaries-build-test-and-development.md), F2 / I-F2,
under accepted R38-02. Review date: 2026-10-14.

The functional outcome is truthful deployment configuration and compatible reuse
of one process-owned resource service supporting the simulator's Rust/Python
workflows. Relevant workloads are repeated and concurrent runtime-handle
acquisition during interactive work and studies. Increasing handle count must
not multiply the configured resource envelope. The change introduces no
model-size, sparsity, stiffness or case-count claim.

Inspected baseline: main `a6c616166948a79231bbd91546d358bf75955c72`, with only the
coordinator's newly allocated, unfilled ADR-0188 untracked.

Coverage includes `pse-runtime` budget/settings/environment/session-factory
owners; `pse-py` settings/runtime acquisition and adjacent registry/workflow
consumers; shared-runtime tests; affected literals; generated Python signatures;
`pse-ids` framing, preimage and encoding hash owners; and blueprint §14.3,
§18.8 and §21. Repository source searches covered `crates/`, `python/`,
`tests/`, `xtask/` and `scripts/`, with a separate whole-workspace search for
the field and parallel-hashing entrypoints.

External downstream clients, whole-runtime architecture, solver algorithms,
scientific adequacy and broader Plan38f implementation are excluded. Public API
compatibility preservation is not a requirement of this authorized design phase.

## 2. Decomposition, ownership and dependencies

The existing responsibility direction is suitable:

- `ResourceBudget` owns validated deployment resource/execution configuration.
- `EngineSettingsInput::resolve` owns default resolution and admission.
- `engine_settings_fields!` declares mechanical native/Python field projections;
  `pse-py` delegates construction to native resolution.
- Python `inspection::runtime::acquire` owns process-runtime compatibility and
  returns the existing `Arc<Runtime>` when budgets match.
- `SharedRuntime::build` constructs the finite engine pool, native cache service,
  CPU admission and math service from the consumed configuration.
- `pse-ids` owns hash algorithms and framing, independently of runtime settings.

Removal stays within those owners and their derived consumers. It requires no
new abstraction, service or dependency.

## 3. Contracts, authority and constraints

**Interface-checked baseline:** `ResourceBudget` derives `PartialEq` and `Eq`.
`inspection::runtime::acquire` compares the complete budget before sharing its
existing runtime. A mismatch returns “all runtime handles share the already
configured process resource budget.”

**Implemented baseline:** the field is declared, transferred during native
resolution and exposed mechanically through Python. The inspected source contains
no branch or hashing call that consumes its value. The dedicated budget test only
sets it and checks successful validation; the shared-runtime policy test combines
it with real timezone/compression/partition settings without asserting a hashing
effect.

**Proposed contract:** process compatibility is determined by the remaining
consumed deployment configuration. Keep memory, spill, reporting, thread,
execution, cache and math settings in the existing equality contract. This review
does not recommend a second, manually maintained list of identity fields.

`SharedRuntime::build` and `session_factory` continue to enforce and propagate
the existing resource configuration. Scientific policies, attempt state, starts,
provider permissions and result identities acquire no new equivalence.

Physical-semantics table and well-posedness statement: not applicable to this
change. No physical quantity, equation, variable role or formulation contract
changes.

## 4. Distinguishing scenarios

| ID | Stimulus / kind | Expected target behavior | Boundary and evidence |
|---|---|---|---|
| S01 | Acquire another runtime handle with identical resolved settings; composition | Reuse the existing process resource service | Existing `runtime::acquire` equality/Arc path; source inspected |
| S02 | Two historical configurations differ only in `hashing_may_use_pool`; contract evolution | The retired distinction is unavailable; their remaining identical configuration is compatible | Intentional expansion of process-runtime compatibility |
| S03 | Change a consumed resource or execution setting; policy | Preserve mismatch rejection against an already configured process runtime | Retain full equality of the remaining budget |
| S04 | Hash the same canonical or encoded bytes; mechanism preservation | Preserve digest bytes, framing and algorithms | Hash owners do not consume the deleted setting |
| S05 | Add concurrent handles under the same budget; composition/growth | Retain one shared pool and CPU/math/cache owners | No new pool or admission route; existing resource tests remain relevant |

For S02, a caller intending isolation is the revealing case. The old flag did
not provide isolation: it caused acquisition refusal. No supported isolation
capability is erased. A future isolation requirement would need its own explicit
resource/lifecycle design rather than an unrelated hashing permission.

## 5. Mechanisms and execution

Deletion removes configuration and projection machinery without changing hashing
placement, passes, bytes or buffers. `pse-ids` uses its existing ordinary BLAKE3
update/framing paths; no `update_rayon` or `update_mmap*` invocation was found in
the searched product/source roots.

The process-runtime mutex, retained runtime, executor and admission lifetimes
remain unchanged. Compatible acquisition returns before new runtime construction.
The target neither combines previously independent pools nor weakens charge,
cancellation or native-drain ownership.

H1, H13, H21 and H27 support preserving consumed guarantees while retiring a
distinction that only obstructs compatible acquisition. No latency, throughput
or capacity improvement is claimed.

Numerical stage columns: not applicable; this change adds or modifies no
formulation, evaluation or solve stage.

## 6. Foundations and gates

These verdicts assess the proposed bounded target, not all baseline runtime code.

| Foundation | Verdict | Evidence / scope |
|---|---|---|
| AP-01 Separation | satisfied | Resource policy, boundary projection and hashing retain distinct owners |
| AP-02 Contracts | satisfied | API removal and expanded compatibility are explicit |
| AP-03 Composition | not applicable | No workflow-composition contract changes |
| AP-04 Domain model/authority | satisfied | Budget retains distinctions actually consumed; native resolution remains authoritative |
| AP-05 Explicit structure | satisfied | Remaining constraints and mismatch rejection stay explicit |
| AP-06 Local reasoning/testability | satisfied | Native settings and hash responsibilities remain independently inspectable; no extra infrastructure is introduced |
| AP-07 Execution fit | satisfied | Shared resource ownership and acquisition lifecycle remain credible; no additional pool or work amplification is introduced |

| Gate | Verdict | Evidence / scope |
|---|---|---|
| G1 Authority | pass | One native declaration/resolution path and derived Python projection |
| G2 Semantic fidelity | pass | Retired public distinction and compatibility change are explicitly selected |
| G3 Validity | pass | Existing budget validation and consumed-setting mismatch rejection are preserved by the target |
| G4 Hidden behavior | pass | Acquisition effects remain at the explicit runtime owner; no getter gains effects |
| G5 Consistency/recovery | pass | One process service and existing resource lifetimes remain unchanged |
| G6 Transformation/reuse | pass | Only the unconsumed field leaves compatibility; consumed policies remain complete |
| G7 Truthful claims | pass | Unsupported hashing permission disappears; no execution or performance claim is added |
| G8 Library leverage | pass | Existing hashing/resource libraries remain; no bespoke generic mechanism is introduced |
| G9 Architectural fitness | pass | Every affected foundation is satisfied |
| PS-G1 | not applicable | No physical-consistency contract changes |
| PS-G2 | not applicable | No structural analysis or solver-admission contract changes |
| PS-G3 | not applicable | No numerical/status/tolerance/post-solve contract changes |

Relevant refinements are DP-01, DP-08, DP-09, DP-16, DP-20, DP-22 and DP-24.
No SHOULD exception or MUST waiver is required.

## 7. Findings

No material defect was found in the proposed target; there are no new actionable
finding IDs.

The baseline issue selected by R38-02 is confirmed, with one correction to its
description: the flag is unused by hashing execution but observable through
runtime compatibility. Describing deletion as behavior-neutral would be
incorrect. This review's decision explicitly includes that behavior change.

An older supporting evidence row in
`r5-capacity-parallelism-boundary.md` describes BLAKE3 parallel hashing as gated
by this field. Current source does not substantiate that statement. It supplies
no missing consumer and must not be used as proof of one.

## 8. Library fit and ownership cost

No new library integration is needed. Existing BLAKE3 ownership remains in
`pse-ids`; native/Python projection ownership remains unchanged. Removing this
application setting does not reject parallel hashing as a library capability.

BLAKE3 feature selection is a separate build decision. The target requires no
feature, version, lockfile or hashing implementation change. Future parallel
hashing would need a concrete consumer and admitted scheduling contract.

## 9. Alternatives and tradeoffs

| Alternative | Assessment |
|---|---|
| Retain the baseline field | Preserves an advertised permission without an execution consumer and an unrelated compatibility distinction |
| Delete field and projections | Selected; simplest sufficient target, with explicit compatibility evolution |
| Retain field but exclude it from equality | Carries the misleading API and adds identity-special-case machinery |
| Wire parallel hashing now | Adds an unrequested execution/scheduling capability requiring separate workload and resource reasoning |
| Introduce an isolation flag | Solves no established requirement in this scope; isolation needs explicit resource ownership |
| Library-owned replacement | None needed: libraries hash bytes; application runtime compatibility remains application meaning |

Reopen the decision if a concrete hashing consumer or a separately authorized
runtime-isolation requirement emerges.

## 10. Verification and limits

| Claim / risk | Evidence | Result / follow-up |
|---|---|---|
| No hashing execution consumer of the field | Interface-checked source/search scope in slot 1 | Confirmed within inspected workspace; external clients not examined |
| Flag affects runtime compatibility | Implemented derived equality and `runtime::acquire` comparison | Confirmed statically |
| Hash bytes remain unchanged | Interface-checked independent hash owners | Target requires no edits to algorithms/framing |
| Removed API absent from imported extension/stub | Proposed | Coordinator verifies after implementation and regeneration |
| Equal remaining budgets share; consumed differences refuse | Proposed preservation | Coordinator selects focused installed-boundary checks |
| Resource envelope remains shared | Interface-checked implementation and existing `shared_runtime` tests | Tests read, not executed |

Compilation, tests, codegen and probes: **not_run** in this read-only target review.
No unit/property model was touched; PS-13 conformance/reference qualification is
outside this change. Static evidence settles the design premise without a probe.
Implementation and generated-stub acceptance remain Plan38f obligations.

## 11. Rule impacts, exceptions and disposition

| ID | Current rule / location | Proposed change | Dependency / if retained |
|---|---|---|---|
| <a id="rc01"></a>RC01 | Public native/Python settings contract in `budget.rs`, `settings.rs`, `inspection/settings.rs` and generated `_native.pyi` | Retire the field, constructor keyword and getter through native declaration and regeneration | R38-02 / S02; retaining it retains the unsupported permission |
| <a id="rc02"></a>RC02 | Complete-budget compatibility in derived `ResourceBudget` equality and `inspection::runtime::acquire`; blueprint §21's conflicting-settings boundary | Document compatibility over the remaining consumed settings and intentional removal of the flag-based conflict | S01–S03; retaining the old distinction requires an explicit identity mechanism and contradicts the selected target |

Blueprint §14.3 and §18.8 resource bounds, shared ownership and native scientific
contracts are preservation constraints, not requested relaxations. Related owner
amendments should clarify the selected boundary without widening this review.

Plan38f owns disposition, implementation and evidence; ADR-0188 and the
coordinator's architecture amendments carry the decision route. This review
does not itself amend authority or mark implementation complete. No exception
record is needed.

## 12. Decision

**Behavioral/semantic adequacy:** Accept at the proposed-design level, including
the intentional runtime-compatibility change.

**Architectural fitness:** Accept for this bounded target; G9 passes.

**Overall decision:** Accept removal without shim or substitute isolation flag.
The strongest evidence is direct inspection of the native declaration,
mechanical Python projection, complete-budget comparison and independent hashing
owners. No material unresolved premise remains for this design choice.

The next obligation is Plan38f implementation and focused acceptance: retire the
selected field/projections/tests/literals, regenerate compiled-PyO3 stubs, verify
the installed boundary and preserve real resource mismatch rejection and hashing
contracts. This decision closes no broader Plan38f packet or product
qualification.

Artifact: `docs/design_review/reviews/design_review_consumed-runtime-settings_2026-10-14.md`.
