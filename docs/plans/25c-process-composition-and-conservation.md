---
title: "25c: Process composition and conservation"
status: draft
date: 2026-09-30
adrs: []
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s02]
---

# 25c: Process composition and conservation

## Context and target

F02/F33 and FU04/FU05 show that scalar wiring, local balance copies and global dynamic
refusals do not provide a composable process model. This plan contributes physical closure
meaning to F16, whose overall disposition and qualification are owned by 25e. R3/R4 govern
connection execution and endpoint obligations.

A material port will declare an independent state specification and transported quantities.
It carries species/index identity, normalization ownership, physical conventions and derived
transport observations. A connection is one semantic occurrence, not a collection of unrelated
scalar equalities. Local storage is independent of the global time route. Conservation is one
authored relationship governing rate formulation and independent temporal qualification.

## Decisions and consumed contracts

1. **State independence.** A state specification identifies independent coordinates and the
   normalization/reconstruction operation for dependent composition coordinates. Fully supplied
   boundary states are checked for consistency; a connection does not add another normalization
   equation or equate every exported property.
2. **Connection compatibility.** Compatibility is determined by consumed state/transport
   contracts, not equal property-package IDs. Direct connections bind independent state once
   and carry transport-agreement obligations. Unlike contexts require an explicit translator
   stating conserved component/energy transport, pressure policy and target-state reconstruction.
   Equal FTPz types alone do not establish energy consistency.
3. **Orientation and boundaries.** Consume A4's owner-relative signs. Each connection has
   stable endpoint and occurrence identities; internal transported contributions cancel once,
   while external boundaries form the flowsheet conservation obligations.
4. **Local accumulation.** Indexed inlet/outlet sets supply shared control-volume balance
   operations. Constitutive splitting, pressure and equilibrium laws remain unit-owned.
   A local storage declaration adds inventories and initial conditions. Memoryless units
   contribute algebraic balances during dynamics without requiring fictitious holdup.
5. **Executable initialization.** Admit each scheduled unit as an explicit map or a conditional
   equation solve. Conditional admission checks inputs, residual ownership, outputs, structure
   and selected solver capability. Connection topology supplies tear candidates but never
   proves causal executability. Simultaneous initialization is an explicit alternative, not
   a silent fallback after failed unit-local admission.
6. **Temporal conservation.** Declare inventory, original signed flux/source contributions,
   tolerance and permitted event transfers together. Formulate rate equations and independently
   evaluate accumulated flux/reset closure from that meaning. Never verify conservation by
   integrating the solved derivative itself. General authored expressions and state-dependent
   transfers require an extension beyond the native balance's current single-state/constant form.
7. **Qualification split.** This plan owns closure descriptors and original-space evaluation.
   25e owns whether required closure and actual endpoint coverage permit a result. Physical
   checks must retain their actual Closure kind and evidence, not appear as NotRequired.

A scalar-expansion macro was rejected because it would retain per-scalar topology and duplicate
tear/closure semantics. A new project-owned recycle solver is unnecessary: retain existing graph,
tear-selection, nonlinear-root and integration library owners.

## Packets

| Packet | Prerequisites | Responsibility | Status |
|---|---|---|---|
| <a id="c1"></a>C1 State and aggregate ports | A2/A4; H2 expression/admission contract | Declare independent state, transported observations and indexed groups | planned |
| <a id="c2"></a>C2 Connections and boundaries | C1; I1 identity framing | Lower one connection occurrence and its topology/transport obligations | planned |
| <a id="c3"></a>C3 Indexed control volumes and storage | C1/C2; B1 for element-total consumers; B2 for reactive consumers | Reuse balance operations across multi-port units; separate local storage from time route | planned |
| <a id="c4"></a>C4 Executable initialization | C2/C3; E1/E2; D1/D3 root capabilities | Admit conditional unit solves and connection tears; migrate recycle initialization | planned |
| <a id="c5"></a>C5 Temporal conservation and closure | C3; A4 | Lower conserved inventories, original flux and event transfers for both temporal realizations; export descriptors to E4 | planned |

C1 consumes H2's checked occurrence contract, not completion of every H packet. C4 consumes
declared execution from E2; E2 does not depend on C4. E4 consumes C5 later. These are one-way
packet dependencies, not circular requirements that both plans finish first.

### C1/C2 — Connected physical states

**C1 final product.** A state specification declares independent coordinates, their reconstruction/
normalization, species identities, physical conventions and derived transport observations.
A material port binds it to a boundary instance and direction. An FTPz specification can choose
F,T,P and z[j] excluding an explicitly identified reference species, reconstructing that fraction
as 1-sum(z). An FPhz specification makes enthalpy independent and temperature derived; connecting
it cannot impose independent values of both. Full displayed compositions remain checked.
Receiving a connection replaces the supplied-coordinate responsibility rather than adding another
normalization equation. Equipment authors expose one indexed port, not unrelated scalar families.

**C2 final product.** Connection admission consumes the two port contracts and any explicit
translator, returning one occurrence with endpoint identities, semantic index correspondence,
independent-coordinate bindings, visible conversions and transport obligations. Default
correspondence preserves species; missing species/basis needs an explicit admitted embedding,
never positional truncation. Direct connection binds state and assesses derived transport
agreement. A translator reconstructs the target state under its declared component/energy and
pressure contract. The same occurrence supplies the graph edge, diagnostics, tear target and
internal-boundary cancellation. A flowsheet author writes one connection per stream.

Migrate reference unit material ports and their callers as complete connection slices. Retain
legitimate scalar connections for scalar signals; replace scalar families standing in for one
material state. Index species by semantic identity, not vector position. Lower explicit converters
and retain the identities needed for diagnostics, tears and boundary accounting.

Focused acceptance: adding a species changes its indexed declaration/data rather than flowsheet
wiring; reordered species cannot misbind values; wrong basis, missing species or incompatible
context refuses; derived observations do not overdetermine state. Test direct transport agreement
and an explicitly authored unlike-context translator. Delete replaced hand-authored connection
equation families and equation-name tear references when each flow migrates.

### C3 — Balances and local storage

**Implementation vision.** A control-volume operation receives indexed inlet/outlet ports,
material balance basis, unit-owned sources/transfers and local storage declarations; it emits
balance equations and conservation descriptors. Component j obeys
dN[j]/dt = sum(in nflow[j]) - sum(out nflow[j]) + reaction_source[j].
Without storage the left side is zero, including during dynamic analysis. Storage contributes
inventory expressions and initial-condition obligations. Reaction sources remain in total-molar
balances because reactions need not conserve total moles. Element projections consume B's checked
compositions. Adding an inlet changes the indexed collection; pressure/split/equilibrium laws stay
with the unit and no copied generic balance formula is needed.

Generalize control-volume boundaries over indexed inlet/outlet sets. Migrate mixer, separator,
reactor and vessel consumers, preserving their distinct constitutive contributions. Support the
applicable component-total, component-phase, element-total and total material bases explicitly, with attributable
refusal when a particular unit lacks the required physical contract. Element-total admission consumes B1's complete composition evidence.

Focused acceptance composes algebraic mixer → holdup CSTR → algebraic separator in steady,
integrated and simultaneous representations. Only stored quantities add differential states;
missing required initial conditions refuse. Structural generation can be checked in focused
tests; full native journeys execute in 25k. Delete generic balance copies and blanket
dynamic-without-holdup refusals. Legitimate unit-specific contributions remain.

### C4 — Recycle execution

**Implementation vision.** A conditional-unit problem contains an admitted unit submodel,
supplied boundary inputs, owned residuals, required output ports and initialization policy.
Admission proves remaining dependencies are local or declared at that boundary, then obtains E's
structural/capability assessment. Evaluation fixes the inputs under scoped ownership, invokes the
existing admitted solve and returns qualified output-state values or typed refusal. It does not
turn free constrained outlets into fabricated explicit expressions. Recycle iteration tears the
connection's independent state coordinates and repeatedly evaluates these admitted units. Explicit
maps keep their cheaper realization; parent-level coupling that prevents a local solve refuses
before iteration instead of becoming a convergence surprise.

Use the existing FlowGraph/tear mechanism on connection occurrences. Conditional solves fix
declared inputs, solve only the admitted unit residual/output problem and restore temporary
initialization state through existing scoped ownership. External equation coupling that prevents
a local solve is an admission refusal before iteration. Do not synthesize an explicit-map
expression for an implicitly constrained free outlet.

Migrate RecycleFlash to the chosen connection strategy. Focused controls establish graph/tear
identity, conditional admission, selected solver capability and temporary-state restoration.
The coupled recycle journey belongs to 25k. Delete hand-authored equation tears and replaced
initialization orchestration; retain reusable native/library operations.

### C5 — Conservation through time and events

**Implementation vision.** A descriptor identifies conserved subject, inventory I, original signed
flux/source f, reference/basis, tolerance and permitted event transfers J. Its closure is
I(t) - I(t0) - integral(f, t0..t) - sum(J) = 0.
Initialization supplies consistent baseline inventory. Verification evaluates original flux,
excluding the solved accumulation variable. Integration uses inventory evaluation, independent
quadrature and event checks; simultaneous realization assesses equivalent original-space
obligations. Mode changes retain subject identity while switching inventory definitions.
Nonterminal reset checks compare before/after inventories against their transfer. Terminal
events have no reset/impulse; their endpoint uses the terminating mode and active input segment,
including at a coincident scheduled change. Continuous closure applies to integrated and
simultaneous realizations. Reset/terminal settlement applies to the admitted event-capable
integrated route; an event-bearing simultaneous request refuses unless a separately explicit
capability is implemented and qualified. This plan introduces no implicit hybrid simultaneous
solver. E receives the measured closure and coverage facts
and alone decides result permission.

Lower authored inventory/flux/transfers into native balance checks, extending the expression
contract where necessary, and into equivalent original-space checks for simultaneous execution.
Mode transitions name applicable balances and transfers. Retain physical tolerances, source
identities and the independent original-flux oracle.

Focused controls include an unauthorized inventory reset under zero flux, a matching impulse,
a wrong impulse, continuous drift and a mode switch. Preserve existing vessel checks until their
replacement is exercised, then delete the bespoke closure copies in that same migration.
A fixed-horizon integral cannot become an event-truncated integral; E4 decides the declared
endpoint obligations using the descriptors supplied here.

## Authority and handoff

A short ADR establishes connection/state and temporal conservation contracts within the generic
modeling kernel. Update blueprint §10/§12/§13 through the design route, including closure
requirements currently excluded by §10.1. A physical decision change follows A's route instead
of being duplicated here.

Hand off one physical connection/conservation meaning to structure, initialization, qualification,
generated APIs and results. F16 closes only when 25e's verdict/reporting and structural consumers
also migrate; counts of removed accumulators are not the acceptance criterion.

## Execution and evidence

All changes and expected benefits here are **Proposed**. Packet status is planning state;
no implementation or new product qualification is claimed. The [series coordinator](25-design-remediation.md)
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

Not implemented; record actual behavior and evidence labels at closure.

### A mistake made and corrected

Record an actual implementation correction, not a hypothetical planning example.

### Deviations from the plan, deliberate

None recorded. A changed architectural decision follows its owning ADR/design route.
