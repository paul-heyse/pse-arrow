---
title: Library-owned mathematics foundation enhancements and M06–M08 implementation review
date: 2026-09-24
status: complete
scope: M01–M05 enhancements and Plan 14 M06–M08
verdict: Accept scoped implementation
adrs: [ADR-0082, ADR-0083, ADR-0084]
evidence: Tested — 45 targeted library units with explicit force-validation and zero failures; full qualification remains M22
---

# Library-owned mathematics foundation enhancements

## 1. Decision and scope

Accept the implemented M06–M08 packet and the associated M01–M05 improvements.
The supported result is a physically admitted thermodynamic/derivative/assembly
foundation with executable native problem views. It does not yet solve a process
model through a native optimizer. The user-authorized hard pivot governs this review;
remaining Plan 13 scope is not inherited. Formal ADR/blueprint reconciliation remains
on its explicit decision path.

**Method and coverage.** Read the actual provider, typed builder, evaluator, compiler
identity, case binding, sparse assembly, coefficient and native-view implementations.
Use the expanded `symbolica-faer-oximo` and `native-solver-libraries` skills, public
Symbolica/Numerica contracts, exact pinned faer/FeOS/Clarabel interfaces, and targeted
executable controls. Symbolica/Numerica implementation source was not inspected.
The workspace all-target compile checked consumers, including the deleted public
value-artifact interface. Independent physical comparisons, native solver execution,
whole-process memory, performance, persistence, distributed operation and final static
qualification were not exercised; they are not certified by this review.

## 2. Authority and lifecycle map

| Meaning | Authority and derived representations | Mutation/lifetime |
|---|---|---|
| Process/physical semantics | Authored source and admitted quantity registry; `AdmittedBody` retains semantic spec, ordered result quantities and occurrence provenance | Immutable semantic preparation; no retagging of `TypedValue` |
| Provider meaning | `ProviderSpec` with complete physical ports, component order, phase/revision/data identity; embedded FeOS data have recorded origins | Immutable factory; fresh mutable worker per attempt |
| Arithmetic | Symbolica atoms/evaluators and Numerica jets derived after physical admission | `PreparedBody` → `CompiledBody` → independent `Worker` |
| Global layout | Explicit variables, parameters, rows, objective and instance contribution maps | `CaseStructure` is immutable; values do not add membership |
| Sparse numerics | faer canonical structure derived from complete support and bindings | Stable pattern/refill mapping; numeric buffers reset per successful product request |
| Solver classes | Library-derived coefficients or explicit cones; separate variable domains and exact Gram evidence | Coefficient snapshot records structural and fixed/parameter assumptions; no implicit native solve |

Evidence: `pse-compiler/src/typed_math.rs` (`AdmittedBody`, `Request::admit`,
`Request::prepare`); `pse-math/src/binding.rs` (`CaseStructure::new`, `key`);
`pse-kernels/src/lib.rs:302` (`ProviderSpec::key` and complete identity).
All crate paths in this review are relative to `crates/`.

## 3. Semantic contracts and invariants

- Physical admission precedes arithmetic normalization; unchecked public `Stage` and
  `GuardedValueArtifact` construction are removed. Original obligations travel with
  typed outputs even after cancellation. Output demand does not erase an obligation
  belonging to that output or execute one belonging only to another output.
- FeOS coordinates are temperature, molar density and two independent mole fractions.
  Pressure is an output, allowing ordinary density variables/pressure equations.
  NPT vapor/liquid initialization is not a guarantee of branch selection or stability.
- Provider partials and native callbacks use raw derivatives. Numerica's diagonal
  second Taylor coefficients require the explicit factor of two in `JetLayout`.
  Symbolica generates the provider composition; no project differentiation rules run.
- `z = Gx + Bp + c` and explicit row weights determine global assembly. Aliases retain
  local multiplicity. Both ordered cross-partials accumulate on an aliased diagonal;
  the lower triangle contains one ordinary off-diagonal coefficient.
- Source variable representation units determine `x`; body formals and row outputs
  are canonical physical coordinates. Units, bases, datums and point/difference
  semantics are checked at the binding and contribution boundaries.
- A quadratic expression does not establish convexity. A certificate verifies the
  supplied Gram identity using exact represented rationals and nonnegative weights;
  changed coefficients or orientation invalidate that certificate.

## 4. Derivation and execution design

| Stage | RCA §9 computation contract |
|---|---|
| Semantic preparation | Parsed ordered outputs + formal layout + selected finite structure + registry + consumed provider contracts + numerical policy. Symbolica normalization promises real algebra on the admitted original domain, not legacy floating-operation ordering. Missing membership differs from present-empty. Source occurrences remain separate from reuse identity. Finite construction is bounded; no I/O or persistent tracked computation. |
| Compilation | Output ordinals, derivative coordinates/order, optimization options and explicit limits determine a library artifact. Scalar regions have compact inputs; Numerica vectorization and Symbolica-generated provider lifts own arithmetic. Component, operation, scratch and optimizer bounds fail explicitly. Library build cancellation is installed. No JIT/SIMD profile is admitted; no `optimize_stack()` follows vectorization. |
| Thermodynamics | FeOS 0.10.1 PC-SAFT/DIPPR with num-dual 0.14.2; ordered methane/ethane/propane source records, zero binary interactions, fixed explicit-density phase contract and recorded caloric convention. One state per uncached requested order supplies all selected outputs. Cache identity includes input bits, output coverage and order. Invalid inputs/results reject; failure clears cache. Optional NPT stability/regularity checks are outside callback evaluation. |
| Case products | Full selected variables/rows plus local support and physical gathers. faer 0.24.4 canonicalizes sparse coordinates once; adapter maps contributions to canonical slots. Identical body/output/coordinate programs share within preparation; workers remain independent. Complete branch support retains numerical zeros and isolates. Cardinality/native-index/aggregate numeric scratch limits precede acceptance. No graph algorithm or temporal semantics is claimed here; M09 consumes this inventory. |
| Trial execution | Caller supplies finite local inputs and explicit cancellation. Separate objective, constraints, gradient, Jacobian and weighted Hessian demands. Trial cache includes every local input, including guard-only controls. Changing Hessian weights reuses local derivatives and refills global values. Output is returned only after complete finite evaluation; no Arrow/Delta publication or asynchronous revision update. |
| Class projection | Symbolica derivatives establish affine/quadratic degree before bounded expansion/polynomial conversion. Constant and direct-coordinate-bound proofs retain original domain obligations; unsupported proofs reject. Exact Gram evidence, integrality and explicit cones are separate contracts. Snapshots include structural and fixed/parameter identity. NLP/NLE adapters expose separate products; NLE additionally requires finite equalities, square shape and no objective. |

RCA §1/§7 departures from an Arrow/DataFusion-first math pipeline have a concrete
reason: coupled dense local arithmetic, symbolic derivatives and thermodynamic state
work belong to these mathematical libraries. Arrow/DataFusion/Delta retain their
actual data consumers. Salsa adoption is not required by this packet; M10 owns
cross-revision dependency observation and bounded retained artifacts.

## 5. Representative journeys

| Journey | Executed control and result |
|---|---|
| Add a second authored output | `ordered_outputs_share_semantics_across_artifact_profiles`: ordered multi-output preparation shares semantic identity across derivative/optimizer profiles; reversing outputs changes identity |
| Reuse a property in two expressions | `typed_output_demand_coalesces_calls_and_keeps_canceled_obligations`: one call in the active region, exact output/order requests; a canceled property remains fallible, an unrelated output needs no provider call |
| Bind both local factors to one variable | `aliases_repeated_rows_isolates_and_stable_refill`: Hessian 2, repeated row accumulation, retained isolate, stable numeric storage, correct faer JVP |
| Change a parameter at unchanged free variables | `scaled_gathers_factored_quadratics_and_parameter_class_changes`: non-unit gather coefficient, factored polynomial extraction, changed coefficient assumptions and invalidated PSD witness |
| Change multipliers at unchanged physical state | The demand/coalescing control checks changed Hessians with one counted provider evaluation; local derivative caches exclude multipliers and the global refill consumes their current values |
| Invalid thermodynamic trial or interrupted work | `failures_clear_cache_and_workers_are_independent`, derivative budget/cancellation controls, native output atomicity controls; no stale value is published |
| Attempt an unsupported solver class | Negative Gram weights, mismatched certificates, malformed cones/CSC, non-square root layouts and discrete variables in continuous oracles reject |

## 6. Acceptance gates

| Gate | Result in this scope | Evidence/limit |
|---|---|---|
| G1 — Authority | Pass | Typed source/registry, provider spec, case structure and library-derived artifacts have distinct owners. Proposed decision amendments do not silently edit accepted authority. |
| G2 — Semantic fidelity | Pass | Raw/Taylor conversion, mixed partials, aliases, row multiplicity, physical conversion, constants/sense and integrality have targeted controls. |
| G3 — Validity | Pass | Physically sealed construction, request/result validation, finite values, explicit domain obligations, bounded layouts, class/PSD/cone refusal. Actual solver convergence remains outside scope. |
| G4 — Hidden behavior | Pass | Foreign provider work is an explicit fallible region; no global error channel, implicit JIT, phase selection in callbacks or external publication. |
| G5 — Consistency and recovery | Pass | Immutable preparation and isolated attempt scratch; failed evaluation publishes no result. Bound instance identity wraps the original source/provider cause. No durable/revision publication route is claimed. |
| G6 — Transformation and reuse | Pass | Complete provider keys, separate artifact settings, output effects, value-keyed caches, duplicate accumulation and assumption-sensitive coefficient/Gram evidence. Cross-revision incremental reuse remains M10. |
| G7 — Truthful capability claims | Pass | Interpreted math and problem views are implemented; native solving, JIT/SIMD, general domain proofs, empirical EOS qualification and M22 acceptance remain explicitly unavailable. |

## 7. Principle findings

Applicability: semantic authority/types, identity, transformation, local execution,
reuse, provider boundaries, provenance and proportional library use apply. Distributed
revision publication, temporal graphs and native solver algorithms are not implemented
by this packet. Their gates are not transferred from these unit results.

| Finding/status | Principles | Evidence or gap | Consequence | Correction/disposition | Verification |
|---|---|---|---|---|---|
| F01 — Satisfied: unsafe construction is sealed | DM-06, DM-07, DM-24 | `pse-math/src/typed.rs:23`, private fields; `guarded.rs` private `Stage`; `execution.rs:56`, private preparation constructor | Public callers cannot bypass physics with a raw stage program | Replaced the unchecked artifact/API and dependent test | Workspace all-target compile; typed negative controls |
| F02 — Satisfied: callback demand retains semantic effects | DM-24, DM-31, DM-32 | `typed.rs` effect propagation and `execution.rs:808`, effect-aware backward demand | Canceled invalid operations remain invalid, without unrelated property calls | Explicit per-output dependencies; scoped provider coalescing | Counted property/negative-domain unit |
| F03 — Satisfied: library-generated derivatives and correct sparse maps | DM-22, DM-38, DM-41, DM-56 | `jets.rs` Symbolica lift generation; `execution.rs:632`; `assembly.rs` ordered local-pair mapping; `sparse.rs` faer ordering/refill | Mixed partials and aliases cannot silently lose the second contribution | One raw/Taylor boundary and mechanical contribution maps | Raw/mixed derivative and alias/repeated-row controls |
| F04 — Satisfied: solver class claims carry evidence | DM-07, DM-24, DM-42 | `coefficients.rs:49`, degree/domain admission; `coefficients.rs:291`, exact Gram check; native `lib.rs:257`, cone validation | Indefinite Q, changed parameter signs or malformed cones cannot enter a validated convex route | Parameter-sensitive snapshots, exact certificate and explicit cone vocabulary | Scaled parameter-class and conic negative controls |
| F05 — Unresolved outside supported scope: whole-attempt resource and incremental ownership | DM-26, DM-32, RCA §8 | Local artifacts/caches have owners and numeric budgets, but no global artifact store/foreign-allocation accounting or revision scheduler exists | A future large multi-attempt service cannot infer an RSS/concurrency bound from local budgets | M10 must build this from actual consumers; M21/M22 qualify it | Cold/warm/revision/eviction/cancellation and retained-memory cases at those stages |
| F06 — Unresolved outside supported scope: physical operating envelope and native process solution | DM-43, DM-59, DM-60 | FeOS data and derivatives are implemented; the independent reference campaign and native solve workflow are not | A passing unit derivative comparison does not certify all physical states or a solved heater/flash | M11–M18 implement workflows; M20/M22 qualify selected physical references and singular/phase boundaries | Independent original-model residual/property comparisons and solver-backed acceptance |

## 8. Alternatives and architectural leverage

| Alternative | Benefit | Cost and decision |
|---|---|---|
| One generic project AD/thermodynamic/linear-algebra engine | Uniform internal scalar type | Reintroduces the independent mathematics this pivot removes; reject |
| Force Numerica through FeOS and replace all faer usage with Symbolica | Fewer names at the boundary | FeOS requires its compatible native dual trait; numerical sparse duplicate assembly has a direct faer consumer. No equivalent qualified Symbolica sparse refill/JVP profile was established. Reject an unproved scalar bridge. |
| Smaller direct library composition, selected here | Numerica for Symbolica arithmetic, num-dual for FeOS, Symbolica for exact coefficient/Gram checks, faer for numerical sparse storage/JVP | Retain explicit physical/effect/assembly adapters only. No new generic AD, sparse factorization, cone inference or presolve engine. |

Oximo/presolve/native solver crates are available for their later consumers. Their
catalog breadth does not justify adding another model authority in M06–M08.

## 9. Verification and measurement plan

**Tested:** the [execution packet](../../plans/14-m06-m08-execution.md#verification)
records the exact combined unit command: 45 passed, zero failed against baseline
zero, with explicit `pse-relations/force-validate` and the local multi-core Symbolica
license. FeOS derivatives are checked against several finite-difference step sizes
as tests only; no finite-difference runtime fallback exists. `just check` compiled
the whole workspace/all targets. Its dependency future-compatibility notice for
`proc-macro-error2` remains a final-qualification concern, not a project compile error.

**Not measured:** end-to-end latency, retained memory, solver performance or simulator
accuracy. M22 remains the only full integration/component/native-solve/Python/static
qualification stage. No old plan receipt substitutes for those checks.

## 10. Exceptions and unresolved decisions

- The user-authorized library-owned pivot governs the target despite historical
  ordered-IR/DataFusion/Pyomo defaults. ADR-0082–0084 and the
  [foundation amendment](../../plans/14-math-foundation-contract.md#concrete-decision-amendment-prepared-for-the-design-pr)
  name the formal supersession route. Accepted ADRs and the blueprint were not edited.
- The initial EOS package uses predictive zero binary interactions and an explicit
  caloric convention. Do not describe it as a fitted universal mixture package.
- General branch-neighborhood/domain proofs, arbitrary PSD discovery, cone inference,
  JIT/SIMD and full foreign-allocation accounting are excluded. Failure is explicit;
  these are neither approximations nor hidden fallbacks.
- M09 starts from the new complete selected inventories/support. M10 introduces
  semantic incremental ownership; neither should port old Plan 13 code by default.

## 11. Decision and implementation changes

| Priority | Decision | Owner/closure |
|---|---|---|
| P0 | Accept scoped M01–M05 enhancements and M06–M08 implementation | Targeted behavior and consumer compilation complete; packet records exact boundary |
| P1 | Begin M09 from selected case inventories and library support | Structural projection/analysis; preserve isolates, direction and declared semantics |
| P1 | Keep M10 resource/incremental work explicit | No claim of global cache or RSS qualification from local ownership |
| P1 | Complete M11–M18 before advertising native simulation workflows | Native adapters, orchestration, dynamic/fitting profiles |
| P2 | Complete M19–M22 and formal decision amendment | Cleanup, acceptance inventory, full qualification and governance reconciliation |
