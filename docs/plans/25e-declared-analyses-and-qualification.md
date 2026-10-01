---
title: "25e: Declared analyses and qualification"
status: done
date: 2026-09-30
adrs: [ADR-0145, ADR-0146]
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s04]
---

# 25e: Declared analyses and qualification

## Context and target

This plan owns F04/F09/F10/F16/F21/F23/F28, the execution/default portions of F32 and relevant
F35 corrections. It consumes D's F22 operation and C's physical closure work. R4 requires actual
endpoint obligations; R5 keeps root response distinct from optimization KKT analysis.

One Rust-owned declared execution admits route, procedure, intent, solver/initialization policy,
limits, derivative demand and result-use policies. Conformance, Python, studies and durable workers
consume it. One composed decision evaluates independent native stop, original qualification,
physical checks, applicability permission and endpoint coverage. No consumer reconstructs success.

## Decisions and shared interfaces

**Structure and routes.** A bound-view structural assessment carries mode, variable/equation roles,
scope, matching and unmatched identities, plus the count meaningful for that mode. Roots require
complete square original-equation matching. NLP and nonlinear factorable representations require
equality-row matching while retaining optimization freedom. Linear/conic representations declare
their justified native-feasibility treatment. Assess original equations before export introduces
auxiliaries. Reuse the existing structural owner.

Route facts carry intent, auto/explicit selection, derived classes, eligibility/refusal reasons and
the chosen representation/backend. Lexicographic choice is a capability query: Auto uses an admitted
native lexicographic capability when available, otherwise an admitted staged strategy. Explicit
selection of a backend without native lexicographic support uses staged execution on that backend
when each stage is supported; otherwise it refuses. There is no unrequested backend fallback.

**Declared execution.** Route and procedure are distinct. Checking, solving, initialization,
integration and shooting use the authored policy. Inspection/diagnosis consumes the same admission
without executing the procedure. A legacy route argument may not silently change it: remove
redundant arguments; where an explicit override is retained, admit its agreement/compatibility or
refuse. Defaults and requested/applied start choices belong to Rust. The execution plan consumes
D3's canonical profile-to-derivative demand operation instead of declaring another rule.

**Candidate policy.** Default incumbent policy refuses use as a result while preserving lawful
seed-only candidates. AcceptFeasible permits an independently original-feasible, validated candidate
at explicitly supported time/node/iteration/solution limit stops. AcceptWithinGap also requires
a valid original-objective bound and the authored absolute/relative gap criterion. Cancellation,
panic, invalid input, evaluation/numerical failure, memory/resource exhaustion and diagnostic or
relaxed-only points cannot be upgraded.

Keep termination, feasibility/qualification, bound origin/gap and closure facts separate. The
final result/seed/diagnostic/refusal decision carries typed qualifying reasons, allowing an accepted
incumbent and closure opt-in to coexist without a variant for every combination. F consumes the
decision through exhaustive projections; it does not recompute scientific acceptance.

**Closure and applicability.** C declares/evaluates physical closure; B evaluates form/data
applicability. This owner applies the authored policies and reports the checks actually performed.
Required closure is never published NotRequired. Unknown applicability and permitted extrapolation
remain visible in observations and result qualification; permission cannot waive a mathematical
domain or required physical constraint. AllowUnclosed may permit a completed independent
closure assessment outside tolerance, retaining Unclosed and its measured residual. It never
waives original-model constraints or missing/incomplete required closure evidence.

**Endpoints.** Admit FixedHorizon or DeclaredTerminalEvent. The latter identifies the allowed event
and makes applicable endpoint-relative obligations evaluate at the actual endpoint. An unexpected
event is not successful completion. Fixed-domain integrals retain their domain; later required
fit/shooting observations remain unavailable. Qualified trajectory prefix, observation coverage
and final result permission are separate facts. Event termination is not inherently an internal defect.
A terminal event publishes the root state in the mode and active input segment immediately
before termination: no reset, successor mode, impulse or coincident scheduled-input change is
applied. Terminal declarations with resets refuse. Nonterminal samples instead observe the
consistent post-reset successor state, with before/after transfer qualification.

## Packets

| Packet | Prerequisites | Responsibility | Status |
|---|---|---|---|
| <a id="e1"></a>E1 Structural and capability facts | F1 diagnostic contract | Own mode-qualified structure, representation admission, lexicographic choice and route facts | complete |
| <a id="e2"></a>E2 Authoritative declared execution | E1; D4 | Separate route/procedure, centralize defaults/demand/start choices; expose root sensitivity | complete |
| <a id="e3"></a>E3 Composed scientific acceptance | E1/F1; B4; C5 | Apply incumbent, closure and applicability policy once with typed qualifying reasons | complete |
| <a id="e4"></a>E4 Actual trajectory endpoints | E3; C5 | Evaluate declared endpoint and observation/integral obligations | complete |
| <a id="e5"></a>E5 Admitted shooting and result consumers | E2/E3/E4; I4; G3 for persisted facts | Remove bypasses; publish truthful route/structure/qualification facts; migrate all consumers | complete |

D3 implements numerical demand independently of E2's eventual caller migration. E1 is available
before D4, preventing a cycle. E2 precedes C4's initialization consumer; it does not require
completed recycle execution.

### E1 — Structure and capability admission

**Implementation vision.** A StructuralAssessment retains analysis mode, variable/equation roles, original-coordinate
scope, matching witness and unmatched identities. The representation capability names the
admission policy it requires; the result is an attributable admitted/refused assessment, not
just a count. Separately, RouteDecision records derived problem facts, requested auto/explicit
selection, eligible candidates and refusal reasons, selected representation/backend and native
versus staged lexicographic realization. Diagnose, conformance, fitting and solve preparation
read these products. They may present different views, but cannot count or classify the problem
again. A route refusal retains its facts even when no native attempt exists.

Replace caller-specific DOF interpretations and the runtime representation-policy match with the
shared assessment and capability declaration. Preserve named model identities through refusal;
enrich structural diagnostics at the modeling boundary, not only in diagnose_case. Route
selection records refusal reasons even when no solve result exists.

Focused controls cover equivalent consumer assessments, overdetermined nonlinear factorable
equalities refused before SCIP, justified redundant linear/conic treatment, and a stub capability
supporting quadratic lexicographic optimization. Test explicit staged selection and no silent
backend fallback. Delete local coefficient/quadratic route tests and duplicated counts.
Unused whole-model matching/projection deletion is coordinated with H4, not replaced by a second
structural algorithm.

### E2 — Case authority and local analyses

**Implementation vision.** Authored case admission produces a DeclaredExecution containing the temporal route, procedure
payload, intent, resolved solver/initialization settings, limits, derivative request and result-use
policies. A simultaneous route can therefore retain an initialization procedure without being
silently recast as steady. Procedure payloads identify their profile/problem and required inputs;
inspection can expose the plan without invoking it. Entry points resolve the case once and pass
this admitted product to its operation owner. The public initialization request either uses the
authored plan or supplies an explicitly admitted compatible override; Python does not rebuild a
second homotopy/default plan.

Move solver/limit/default interpretation out of conformance into the production owner. Migrate
direct execution, initialization, diagnose/inspect, studies and generated APIs to the same plan.
Root sensitivity uses D4 without a dummy objective; fitting uses the same operation.
Preserve typed requested/applied starts and the distinction between requested procedure and route.

Focused controls show simultaneous and initialized cases retaining their authored choices across
entrypoints, contradictory override refusal, equal omitted/explicit defaults and shared derivative
demand. Delete hard-coded steady specialization, product dependence on conformance helpers,
duplicate route switches and copied initialization policy. J migrates the Python surfaces.

### E3 — Result acceptance

**Implementation vision.** The decision operation consumes independent native evidence (stop, candidate source, original
qualification and any bound/gap), physical closure/applicability observations, requested policies
and coverage facts. It produces final permission, qualifying reasons and refusal reasons while
retaining links to the underlying evidence. A time-limited original-feasible candidate under
Refuse remains seed-only where lawful; under AcceptWithinGap it can become a result only with
a valid bound meeting the selected criterion. Missing bound refuses that opt-in. Missing closure
evidence also refuses even under AllowUnclosed; a completed outside-tolerance assessment may
be explicitly allowed without losing its Unclosed residual. Consumers ask for result/seed
permission from this product, never by comparing a termination spelling.

Compose policies over independent evidence; retain proof origin and typed reasons in the decision.
A certified relaxed bound may help bound the original problem but cannot make an unverified
relaxed point usable. Keep original-coordinate feasibility, domains and required physical checks
as conditions. Default refusal remains explicit rather than encoded by a missing case.

Focused matrices cover every stop category, policy, absent/invalid bound, wrong-source candidate
and combined incumbent/closure opt-in. A valid incumbent with missing required closure evidence
still refuses under AllowUnclosed. Test unknown/outside applicability policy and truthful
closure reporting for vessel, distributed CV and flowsheet obligations. Delete local usable
predicates and termination-based acceptance only as their consumers switch to the shared decision.

### E4 — Endpoint qualification

**Implementation vision.** EndpointRequirement identifies FixedHorizon or the selected terminal event, requested observation
roles/times and integral/check domains. TrajectoryEndpoint records time, state, active mode,
active input segment, quadratures and event/transition identity. Combining them produces the
obligation schedule and coverage assessment: which checks evaluate at this endpoint, which
observations exist, which are inapplicable for this request, and which remain required but missing.
This lets an event-ended trajectory qualify its requested endpoint while a shooting consumer
requiring a later observation still refuses. Terminal root observations use pre-termination
inputs; nonterminal event observations use the settled successor state, with transfer checks
across both sides.

Materialize the actual terminal endpoint as an evaluation point even when it is absent from the
requested output sample grid. Retain its state, mode, cumulative quadratures and active native
input segment; selecting inputs from timestamp alone would use the wrong side of a scheduled
change coincident with termination. Classify future samples according to the admitted endpoint request;
do not silently remove observations required by another consumer. Check event-relative integrals
and temporal transfers at the endpoint.

Focused controls cover an allowed terminal event, an unexpected event, an early fixed-horizon stop,
event-relative versus fixed-domain integrals, coincident scheduled-input changes, terminal-reset
admission refusal, nonterminal reset/impulse settlement and missing downstream observations. Delete
Completed-only shortcuts and Event-to-internal-error mappings. Full integrated/simultaneous
journeys execute once in K3.

### E5 — Admitted completion and published facts

**Implementation vision.** Shooting preparation produces an admitted execution request, then an owned start/handle using
the same CPU/memory/cancellation supervision as other native work. Completion combines its
native evidence with continuity, observation and closure checks through E3/E4. The returned
result links the declared execution, route/structural assessments and final decision. Persistence
and generated APIs expose those typed products, including absent candidate or refusal facts;
there is no shooting-specific success boolean inferred from its raw solver report.

Expose shooting through admitted asynchronous start/handle execution; make raw synchronous
unadmitted solve internal. Continuity, sample coverage and physical checks feed E3/E4. Conformance
reads the resulting acceptance rather than the native termination category.

Persist route intent, auto/explicit choice, derived classes and admission reasons through G's
versioned schema route. Diagnostic-only refusals need route facts without a fabricated successful
run. F carries typed failure envelopes; J generates transport. Correct F35's numerical-source
inventory, structural diagnostic attribution and obsolete owner references alongside their owners.

Focused controls include qualified Acceptable versus unqualified Success, shooting's resource
admission, and serialized facts sufficient to explain the decision. Delete bypass entrypoints,
consumer acceptance copies and prose-only decision projections.

## Authority and handoff

Amend ADR-0106 item 11 and blueprint §16.6 for incumbent/endpoint policy; update ADR-0111's
explicit lexicographic selection rule. Structural modes and root response update §15.2/§15.5.1;
C's closure contract updates §10.1; route/procedure changes follow the registry/kernel contract
route. Amend §13/§19 and correct F35's §5.1/§15.3/§16.2 statements with their actual owners.
ADRs describe each enduring decision, not one record per finding.

F consumes scientific decisions and supplies detailed diagnostic projections. G persists their
typed meaning; J exposes it; K qualifies the assembled behavior. F16 closes only after C's
conservation descriptors and every structural/closure consumer here are migrated.

## Consumed 25c prerequisite slice

**Implemented/Tested, 2026-10-01; scoped focused verification recorded in [25c Verification](25c-process-composition-and-conservation.md#verification):** Conditional admission retains complete owned rows/unknowns and dependency/structural witness, selects solver capability before iteration, and routes work through the existing runner. C5 supplies original physical Closure facts and stable inventory descriptors; shooting consumers stitch these facts against one baseline. Unified analysis/route, result permission and actual endpoint policy remain open; F16 is not closed by C5. The maintainer authorized only this required slice and its complete affected consumer migration. This packet remains partial; [25c](25c-process-composition-and-conservation.md) owns the slice evidence.

## Consumed 25d prerequisite slice

**Implemented/Tested, 2026-10-01:** E1/E2's required mathematical-realization/response
slice is integrated. Original square response admission preserves the complete original equality/state inventory and library matching, including isolated coordinates, separately from numerical rank. E2 consumes actual derivative demand and exposes qualified Root parameter response through the existing request/result tables; optional unavailable response retains a qualified base root. Broader representation/lexicographic admission, route/default/start ownership and result-permission obligations remain open.
[25d Verification](25d-mathematical-realization-and-response.md#verification) owns commands,
conditions, composite results and limits; this does not close the enclosing packets.

## Execution checkpoint — 2026-10-01

E1–E5 and their required F/G/I/J slices are implemented. The clean starting baseline was
`88b653d7d977acfc1d80e41d337835e4843ff209`; the concurrent governance commit `bc4e0d9b`
and its architecture revision were preserved. ADR-0145/0146 remain proposed pending the
decision-PR route. Continue with the remaining companion packets; broad qualification stays
in 25k. The existing development database was neither reset nor migrated.

## Verification

**Tested, 2026-10-01:** Focused controls ran locally on Linux with the pinned nightly,
locked dependencies and explicit `pse-relations/force-validate`. Failure baseline: zero.
Native scientific commands use the licensed `direnv exec .` environment, recipe-owned solver
libraries, the default 120 GiB process memory cap and one test thread. Generated-contract and
pure admission tests require no native scientific execution. These are mechanism controls;
they do not qualify the complete reference set, durable worker journeys or the series.

| Command | Result against zero failures | Established scope |
|---|---|---|
| `just unit-package pse-authoring 'test(fixture_route_procedure_endpoint_roundtrip_and_legacy_refusal)' --test-threads 1` | 1/1 passed | Distinct authored route/procedure/endpoint round trip; replaced combined syntax refuses |
| `just unit-package pse-modeling 'test(declared_policy_defaults_and_independent_procedure_admission) or test(native_fixture_options_admit_only_exact_primitives)' --test-threads 1` | 2/2 composite passed after correcting one malformed test input | Defaults, compatible procedure/route combinations, native primitive settings and contradictory declaration refusal |
| `just unit-package pse-model 'test(policy_identity_tests::)' --test-threads 1` | 1/1 passed | Independent incumbent policy changes numerical request identity |
| `just unit-package pse-backend-native 'test(declared_lexicographic_capability_native_staged_and_no_fallback) or test(retained_original_assessment_distinguishes_matching_and_native_feasibility)' --test-threads 1` | 2/2 passed | Stub quadratic native priorities, explicit staged execution/no fallback, original matching versus justified native feasibility |
| `just unit-package pse-backend-native 'test(certify_routes_to_scip_when_linked)' --test-threads 1` | 1/1 passed | Local adapters retain typed certification refusal |
| `just unit-package pse-runtime 'test(workflow::numerics::tests::) or test(math::staged::admission_tests::)' --test-threads 1` | 10/10 passed | Six stop/bound/qualification/closure/applicability/coverage controls and four deterministic CPU permit abandonment/cancellation/panic-cleanup/reuse controls |
| `just unit-package pse-runtime 'test(adapter_not_linked_is_unsupported)' --test-threads 1` | 1/1 passed | Empty adapter inventory and unavailable explicit selection retain route refusal facts |
| `direnv exec . just unit-native-package pse-backend-native pse-backend-native/diffsol,pse-backend-native/idas 'test(terminal_endpoint_is_off_grid_and_keeps_pre_change_inputs)' --test-threads 1` | 1/1 passed, both methods | Off-grid root, pre-change inputs at coincident scheduled change, prefix/missing observations, wrong event and fixed-horizon refusal |
| `direnv exec . just unit-native-package pse-runtime pse-runtime/solver-ipopt,pse-runtime/solver-diffsol,pse-runtime/solver-idas 'test(declared_terminal_endpoint_qualifies_conservation_prefix_without_fabricating_integrals)' --test-threads 1` | 1/1 composite passed, both integrators and three coverage cases | Actual endpoint closure, physical endpoint export, terminal-reset admission refusal, incompatible endpoint override and unavailable whole-domain integral report |
| `direnv exec . just unit-native-package pse-runtime pse-runtime/solver-ipopt,pse-runtime/solver-diffsol,pse-runtime/solver-idas 'test(conformance_publishes_original_structural_refusal_without_a_run_result) or test(overspecified_root_is_refused_structurally_naming_members) or test(declared_simultaneous_initialization_retains_procedure_and_admits_overrides_once) or test(shooting_matches_simultaneous_optimum) or test(multiple_shooting_continuity_closes) or test(authored_shooting_fixture_solves) or test(kernel_starts_numerics_and_constant_solver_share_the_existing_pipeline)' --test-threads 1` | 7/7 passed | Typed original refusal with retained cause/model paths; simultaneous initialization and overrides; three supervised shooting controls including optimizer-bypass refusal; published admission for a constant candidate whose final model checks refuse |
| `just db-test 'package(pse-operations) & test(migration_)' --test-threads 1` | 8/8 composite passed after six initial fixture/canonical-layout failures were corrected; final frozen-target run 8/8 | PostgreSQL 18 exact predecessor transitions, preserved catalog/payload/lease/retention records, both history checksum conflicts, drift refusal without mutation, actual interrupted commit/resume, active and closed-but-borrowed generations, concurrent/repeated admission |
| `just unit-package pse-codegen 'package(pse-codegen) & test(codegen::postgres::tests)'` | 4/4 passed | Generated component ownership and canonical PostgreSQL declarations |
| `direnv exec . just py-unit-native python/pse/tests/test_modeling_kernel.py::test_conformance_runs_the_declared_reference_set python/pse/tests/test_modeling_kernel.py::test_modeling_authored_fixture_shared_checks_and_owned_tables` | 2/2 composite passed after two stale consumer expectations were corrected | Synthetic pure conformance CLI exports; generated owned admission streams; Python simultaneous initialization/inspection/direct execution/study agreement and procedure refusal |

**Interface-checked, 2026-10-01:** `just check-package pse-runtime` and
`just check-package pse-operations` passed. `direnv exec . just check-solver-contracts`
passed on the final source: backend/runtime/compiler/relations all-target compilation with
`pse-runtime/native-solvers,pse-relations/force-validate`. Initial native checks exposed source
and stale test consumer errors; all were repaired, including 13 errors in the final all-target
check. Current project compile errors/warnings: zero; one dependency future-incompatibility
notice remains for `proc-macro-error2 v2.0.1`. This is a composite receipt, not an initially clean run.

**Implemented:** `direnv exec . just codegen` passed for all six schema targets, physical
fixture outputs, native bindings and hakari; `direnv exec . just py-sync-native` refreshed the
editable linked boundary and generated native stubs from actual compiled PyO3 metadata.
The first scientific runtime selection without the licensed environment failed 0/5 and exited
abnormally; loading the checkout environment corrected that prerequisite. One Python invocation
collected no tests because the recipe did not preserve a spaced selector; explicit node IDs
were used for the reported result. Endpoint test input corrections supplied the required
physical quadrature tolerance and asserted reset refusal at its earlier admission boundary.

Full integration, reference/native adapter campaigns, broad Python/parity, formatting, hygiene,
governance, docs publishing and performance measurement are **not_run** here: the series assigns
them once to [25k](25k-integrated-qualification-and-closure.md). No performance or whole-product
qualification claim is made. Review M1–M3 in the bounded E1/E2/E4 source follow-up were corrected;
that review excludes the executor's own E3/I4 work, G3, Python and a formal architecture verdict.

## Outcome (recorded after implementation)

### What was built

**Implemented:** E1–E5 are complete. Original mode-qualified structural assessments and
capability-owned route/priority decisions survive success and refusal. One declared execution
retains independent temporal route/procedure and Rust-owned settings, derivative demand and
initialization defaults across conformance, Python, study and durable consumers. Typed candidate
qualifiers/refusals compose independent original feasibility, native termination/bound origin,
closure, applicability and endpoint/coverage evidence. Native stop spelling cannot grant permission.

Actual terminal endpoints carry physical state, mode, cumulative quadratures and the live input
side independently of requested samples. Conservation evaluates the qualified prefix; fixed-domain
integrals and later observations retain their required extent. Shooting runs through admitted
asynchronous supervision and publishes joined native/scientific completion. Canonical provider
registration keys remain stable when attempt workers restrict derivative order.

**Implemented/Tested:** I4's dispatched work owns CPU capacity through required teardown;
G3's required operational slice splits component identities/histories and performs explicit,
checksum-bound, quiescent preserving transitions from the exact known predecessor. Runtime-ready
admission verifies domains, columns, constraints, defaults, indexes and both histories. Closed pools
retain the generation lease until borrowed clients drain. Historical records remain intact;
missing newer scientific evidence is never manufactured. Wider directional interpretation,
migration planning/report surfaces and orphan reconciliation remain 25g obligations.

Replaced combined fixture execution fields/syntax, redundant route arguments and study route
storage, Python initialization defaults, copied conformance execution policy, local candidate-use
predicates and public raw shooting solve were deleted. Registry-generated Rust/Python/SQL/stubs
carry the new facts. Architectural owners and revision 92 record the resulting contracts and
F35's numerical-source inventory, actual identity owner and structural model-path attribution.
No compatibility execution path or destructive upgrade was retained.

### A mistake made and corrected

Attempt worker creation re-keyed restricted provider registrations from their changed descriptors,
losing canonical keys referenced by compiled operations. Reusing the shared attempt-worker constructor
preserved admitted keys in dynamics, endpoint checks and fitting. A second endpoint transport defect
leaked synthetic shooting anchors into physical checks; projecting actual endpoint inputs back to
original integration coordinates repaired the failure without weakening checks. The focused shooting
and endpoint controls now pass.

### Deviations from the plan, deliberate

G3 was pulled forward only as needed to preserve changed persisted contracts; it does not claim
completion of 25g's directional reader, future migration-planning/report or orphan lifecycle scope.
The bounded architecture review remains an author judgment because runtime agent limits prevented
a separate reviewer. An executor independently assessed untouched E1/E2/E4 source; its M1–M3
findings were repaired and targeted tests passed. ADR status acceptance and full series qualification
remain separate from functional plan closure.
