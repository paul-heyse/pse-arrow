---
title: "25e: Declared analyses and qualification"
status: in-progress
date: 2026-09-30
adrs: []
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
| <a id="e1"></a>E1 Structural and capability facts | F1 diagnostic contract | Own mode-qualified structure, representation admission, lexicographic choice and route facts | partial: 25c/25d prerequisite slices |
| <a id="e2"></a>E2 Authoritative declared execution | E1; D4 | Separate route/procedure, centralize defaults/demand/start choices; expose root sensitivity | partial: 25c/25d prerequisite slices |
| <a id="e3"></a>E3 Composed scientific acceptance | E1/F1; B4; C5 | Apply incumbent, closure and applicability policy once with typed qualifying reasons | planned |
| <a id="e4"></a>E4 Actual trajectory endpoints | E3; C5 | Evaluate declared endpoint and observation/integral obligations | planned |
| <a id="e5"></a>E5 Admitted shooting and result consumers | E2/E3/E4; I4; G3 for persisted facts | Remove bypasses; publish truthful route/structure/qualification facts; migrate all consumers | partial: 25c prerequisite slice |

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

## Execution and evidence

The consumed 25c/25d prerequisite slices above are **Implemented**; their focused evidence
is owned by the linked plans. The remaining packet scope and expected benefits are **Proposed**. No full-packet
completion or new broad product qualification is claimed. The [series coordinator](25-design-remediation.md)
owns finding dispositions and decision dependencies. Packets compile affected owners, run focused
behavioral checks with explicit force-validation, regenerate changed declarations, and immediately
delete replaced code, callers, obsolete tests and fixtures. No shims or parallel production paths remain.
Full integration, formatting, lint and performance qualification run once in
[25k](25k-integrated-qualification-and-closure.md), after the series' functional scope is complete.

Use current recipe-owned checks such as `just check-package <pkg>` and
`just unit-package <pkg> <filter>`; select isolated tests rather than broad suites hidden under
a unit label. The acceptance scenarios above define what those tests must establish, not claims
that tests with particular names already exist. Cross-owner scientific/storage journeys are authored
with the functional work and executed in 25k. Record state, decisions and next steps during work;
record actual commands, conditions and failures against zero in the final qualification evidence.

## Outcome (recorded after implementation)

### What was built

Full-plan closure remains outstanding. The implemented 25c/25d prerequisite slices and
their remaining boundaries are recorded above; the linked plans own their focused evidence.

### A mistake made and corrected

Record an actual implementation correction, not a hypothetical planning example.

### Deviations from the plan, deliberate

None recorded. A changed architectural decision follows its owning ADR/design route.
