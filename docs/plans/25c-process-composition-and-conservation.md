---
title: "25c: Process composition and conservation"
status: in-progress
date: 2026-09-30
adrs: [ADR-0142]
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
| <a id="c1"></a>C1 State and aggregate ports | A2/A4; H2 expression/admission contract | Declare independent state, transported observations and indexed groups | implemented; focused tests passed; final workspace compile pending |
| <a id="c2"></a>C2 Connections and boundaries | C1; I1 identity framing | Lower one connection occurrence and its topology/transport obligations | implemented; focused tests passed; final workspace compile pending |
| <a id="c3"></a>C3 Indexed control volumes and storage | C1/C2; B1 for element-total consumers; B2 for reactive consumers | Reuse balance operations across multi-port units; separate local storage from time route | implemented; focused tests passed; final workspace compile pending |
| <a id="c4"></a>C4 Executable initialization | C2/C3; E1/E2; D1/D3 root capabilities | Admit conditional unit solves and connection tears; migrate recycle initialization | implemented; focused tests passed; final workspace compile pending |
| <a id="c5"></a>C5 Temporal conservation and closure | C3; A4 | Lower conserved inventories, original flux and event transfers for both temporal realizations; export descriptors to E4 | implemented; focused tests passed; final workspace compile pending |

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

## Execution checkpoint

Restart checkpoint, 2026-10-01. Implementation starts from the completed 25a/25b tree
at `e38eaed6`. The maintainer authorized C1–C5 and only the required external prerequisite
contract slices, including their complete affected consumer migrations. All agents have
concluded; their source changes are integrated in the existing `main` checkout.

C1–C5 functional implementation, reference-consumer migration and deletion obligations
are complete. Focused controls and full `just codegen` have passed with the composite
verification recorded below. The final affine boundary correction is implemented and
tested: the compiler owns the actual subtraction, the solver row is an interval, point
inputs/outputs remain points, and explicit numerical magnitude policy is preserved.
The scaled point-inventory and actual-reference controls also passed after test-only
physical-type/resource repairs. No functional worker task or test command remains active.

The plan stays **in-progress** for the root's final workspace compilation and closure
handoff. At the maintainer's restart request, no new full workspace build was started.

### Remaining work and resume order

1. Run `source .envrc.local; just check` in this same checkout to compile the complete
   workspace and all targets on the pinned toolchain. Repair actual compile failures,
   if any, and rerun the affected targeted controls. Record the command and actual
   warnings/errors against the zero target. The existing focused receipts need not
   be repeated wholesale unless a repair changes their exercised contracts.
2. Complete the root acceptance/handoff: update the coordinator's finding dispositions
   for F02/F33/FU04/FU05 from this evidence, leave F16's E-owned obligations open, and
   mark 25c done after the remaining compile check succeeds. Retain this record while
   the active series and prerequisite plans depend on it.
3. ADR-0142 remains proposed pending its decision PR; architecture revision 89 records
   the maintainer-authorized implementation. Follow the decision route separately;
   implementation evidence does not accept an ADR.

Full native/Python coupled journeys, integration/component/parity campaigns, performance
measurements and manual static qualification remain 25k work. The wider D/E/F/H/I plans
remain partial. In particular, broader H2 still owns legacy parse-at-use consumers,
shared continuous/domain/rewrite helpers, prior authored-body consumers and full
role/position propagation of A3 admissions. Required checked process fields already
consume retained AST occurrences; this checkpoint does not claim full parse elimination.

## Execution and evidence

The target decisions are **Proposed** in ADR-0142; functional implementation and focused
verification are complete; final workspace compilation and closure handoff remain. No series integration or new broad product qualification is claimed. The [series coordinator](25-design-remediation.md)
owns finding dispositions and decision dependencies. Packets compile affected owners, run focused
behavioral checks with explicit force-validation, regenerate changed declarations, and immediately
delete replaced code, callers, obsolete tests and fixtures. No shims or parallel production paths remain.
Full integration, formatting, lint and performance qualification run once in
[25k](25k-integrated-qualification-and-closure.md), after the series' functional scope is complete.

Use current recipe-owned checks such as `just check-package <pkg>` and
`just unit-package <pkg> <filter>`; select isolated tests rather than broad suites hidden under
a unit label. The acceptance scenarios above define the behavioral scope; Verification records the
focused checks actually executed and their limits. Cross-owner scientific/storage journeys are authored
with the functional work and executed in 25k. Record state, decisions and next steps during work;
record actual commands, conditions and failures against zero in the final qualification evidence.

## Verification

**Tested, 2026-10-01:** Focused controls run locally on Linux with the pinned toolchain;
every test recipe explicitly enables `pse-relations/force-validate`. The failure baseline
is zero. Licensed mathematical controls silently load `.envrc.local`. Native recipes apply
the repository's 120 GiB process memory cap; compiler/modeling mathematical selections
use `bash scripts/memory-cap.sh` as shown below. Numeric selections use one test thread.
These controls exercise individual admission/lowering/execution mechanisms rather than a
full recycle, thermodynamic, Python or scientific campaign.

| Command | Result against zero failures | Established scope |
|---|---|---|
| `just unit-package pse-authoring 'test(process_contract_state_material_port_connection_and_inventory_roundtrip)'` | **Tested:** 1/1 passed | Parser/render round trip for the new declarations; no licensed mathematics |
| `source .envrc.local; bash scripts/memory-cap.sh just unit-package pse-modeling 'test(checked_occurrences) \| test(process_contract)' --test-threads 1` | **Tested:** 22/22 passed | Checked lexical/index/physical occurrences, independent state and semantic correspondence, connection overrides, translators and refusal controls |
| `source .envrc.local; bash scripts/memory-cap.sh just unit-package pse-quantity 'test(component_flow_time_integrals_keep_species_and_molar_contracts)' --test-threads 1` | **Tested:** 1/1 passed | Both ComponentFlow × Time orders preserve species/molar Amount; TotalAmount stays distinct and wrong basis/subject cannot cross the Amount boundary |
| `source .envrc.local; bash scripts/memory-cap.sh just unit-package pse-math 'test(numerical_projection_preserves_precedence_and_affine_magnitudes) \| test(numerical_difference_projection_preserves_affine_policy_and_refuses_wrong_contracts)' --test-threads 1` | **Tested:** 2/2 passed | Explicit Fahrenheit numerical magnitude policy and provenance survive coordinate/difference projection; normalized physical residual is invariant and wrong target contracts refuse |
| `source .envrc.local; bash scripts/memory-cap.sh just unit-package pse-compiler 'test(conditional_unit) \| test(document_reuse_decoded_interpretation_changes_match_clean_admission) \| test(indexed_process_composition) \| test(authored_vessel_temporal_) \| test(indexed_balance_)' --test-threads 1` | **Tested:** 8/8 composite passed on the final affine snapshot (7 plus the reuse retry) | Mixer → stored CSTR → memoryless Separator in steady/integrated/simultaneous routes; four material bases; original vessel closure and drift; local conditional inventory, original recycle graph and decoded-context reuse |
| `source .envrc.local; just unit-native-package pse-backend-native pse-backend-native/diffsol,pse-backend-native/idas 'test(conservation::) \| test(conservation_functions_share_original_derivative_diagnostics) \| test(dynamics::anchored::tests::) \| test(dynamics::tests::process::) \| test(idas_scheduled_inputs_with_recoverable_trials) \| test(state_triggered_reset_sensitivity_includes_moving_event_and_dae_consistency) \| test(scheduled_input_sensitivity)' --test-threads 1` | **Tested:** 18/18 passed | Independent continuous closure, wrong/matching impulses, drift, mode changes, terminal/coincident scheduled settlement and continuing reset sensitivities on both native adapters |
| `source .envrc.local; just unit-native-package pse-backend-native pse-backend-native/diffsol,pse-backend-native/idas 'test(conditional_unit_derivative_demand_comes_from_adapter_capability)' --test-threads 1` | **Tested:** 1/1 passed | Conditional derivative demand comes from the selected adapter capability |
| `source .envrc.local; just unit-native-package pse-runtime pse-runtime/solver-diffsol,pse-runtime/solver-idas,pse-runtime/solver-kinsol 'test(authored_conservation_) \| test(conservation_stitch) \| test(document_reuse) \| test(conditional_unit)' --test-threads 1` | **Tested:** 20/20 composite passed (18 plus two repaired fixture controls) | Eight authored conservation controls on Diffsol/IDAS, two shooting controls, four document-reuse controls and six conditional-unit controls, including affine policy preservation and actual-reference admission |

**Implemented:** `source .envrc.local; just codegen` passed after the final identity-frame
changes: all six schema targets, generated physical operations, Python candidate validation
and native bindings completed through their generators. **Not run:** Final `just check`
workspace/all-target compilation remains the first restart step.

The final compiler selection passed seven controls; its last reuse control could not start
because its test executable disappeared during the run (ENOENT). The same licensed,
memory-capped recipe with filter
`test(document_reuse_decoded_interpretation_changes_match_clean_admission)` passed 1/1
after recreating artifacts. This is a composite 8/8 result, not an initially clean run.
The final runtime selection passed 18/20. Its two failures were test-fixture admission:
an additive rate incorrectly declared Difference, and insufficient reference evaluator
storage. After correcting the fixture rate to Point while retaining the explicit
TemperatureRate × Time → DeltaTemperature operation, and declaring 256 MiB evaluator
storage, the same native recipe with filter
`test(authored_conservation_point_inventory_) | test(conditional_unit_reference_request_)`
passed 2/2. No production acceptance condition was relaxed. The final native run also
reported an unused `RelationRow` import in the p09 test and a `proc-macro-error2` future
compatibility warning; no warning-free workspace claim is made.

Earlier focused selections exposed initial-coordinate, semantic guard, lexical/index scope,
physical inverse and reference-fixture defects; they were repaired under their owning
contracts. The eventual focused evidence is composite, not a claim that the first run passed.
The nonlinear original-flux control uses a physical closure tolerance of `1e-5 s` and
initial step `0.001`; a tighter `1e-6 s` request exposed a Diffsol defect and is not claimed
as qualified. The affine inventory control uses a named rate with canonical K/min and an
authored K/s flux, testing one conversion while retaining temperature-point inventory and
interval-valued transfer meaning. The reference recycle admission control retains the
original source and coherent binding selection with synthetic test-only property data;
it has 256 MiB evaluator storage, a 1 GiB workspace and 4 GiB runtime pool and establishes structure/capability
refusal before iteration, not a scientific oracle or a converged recycle.

**Not run:** Full native/Python coupled journeys, integration/component suites, parity,
performance measurements and manual static qualification remain assigned to 25k. Formatting
and hygiene remain the end-of-turn hooks' responsibility. No new whole-product qualification
or **Measured** claim is made. ADR-0142 remains **Proposed** pending its decision PR.

## Outcome (recorded after implementation)

### What was built

**Implemented:** State specifications declare semantic-indexed independent coordinates,
one dependent reconstruction and original transported observations. Material ports consume
these specifications. One connection occurrence supplies coordinate bindings, physical
transport obligations and graph/tear identity; connection overrides retain the occurrence
identity. Direct admission checks full physical contracts and complete species/role coverage.
An ordinary authored translator owns its separate state contracts and conservation laws.
The reference thermodynamic, aqueous, Feed/Product, control-volume, Mixer, Separator, Flash,
PFR and recycle consumers have migrated to aggregate material boundaries. Scalar signal
connections retain their existing purpose.

**Implemented:** Shared indexed balance operations cover component-total, component-phase,
element-total and total material bases, retaining unit-owned reaction, pressure, equilibrium
and partition laws. Memoryless units remain algebraic in every time route. Storage adds
original inventories and initial conditions. The current control-volume storage projection
supports single-phase component-total/component-phase inventories; element-total/total
storage refuses with an attributable request for an explicit inventory projection. Separator
observations independently require physical closure without adding redundant equations.

**Implemented:** Each causal unit request declares an explicit map or an owned conditional
root problem. The compiler derives one inventory of local residuals and unknowns, checks
external dependencies and structural matching, and consumes the selected adapter's derivative
requirement. The existing native root runner executes conditional problems with temporary
input overlays restored after every call. Derived member inputs retain their original
constituents and physical conversion. Their boundary residual is an actual compiled typed
subtraction, so affine point observations retain point inputs/outputs and an interval
residual. Numerical projection preserves resolved tolerances, characteristic magnitudes
and provenance, including noncanonical representations. Typed diagnostic envelopes preserve source identities,
the mathematical cause and its actual boundary class. Two-sided bounds and model-level
implicit handlers currently refuse local conditional realization before iteration, with
simultaneous initialization available as an explicit alternative. RecycleFlash uses five
aggregate stream occurrences and an authored, typed conditional handoff fixture; the coupled
solve is reserved for 25k.

**Implemented:** `conserve` lowers original inventory, signed flux, physical tolerance and
allowed guard-bound transfers to integrated and simultaneous descriptors. Diffsol and IDAS
evaluate actual original inventory and flux, independently accumulate flux, and assess actual
mode/input/event boundaries. Transfer declarations bind semantic guard members in the owning
instance, including nested reuse. Composite inventories preserve original coordinate initial
conditions as required original-space checks. Continuous drift, unauthorized/wrong transfers
and inconsistent original initialization cannot become accepted closure. Shooting retains one
global baseline and rebases cumulative flux/transfers when stitching windows. Terminal events
retain the terminating mode/input side and perform no reset; event-bearing simultaneous
requests refuse in this scope. Required Closure facts are supplied to the existing result
consumer without claiming completion of E's unified qualification policy.

**Implemented:** Required H/I prerequisites retain checked process expression occurrences
with lexical/physical/index context, reuse document interpretation under the complete consumed
context, and frame new process slots through the identity owner. The consumed D/E/F/H/I
slices and their remaining scope are recorded in their own plans. All registry, Rust, Python and documentation surfaces were regenerated through
`just codegen`, including the species/molar ComponentFlow × Time → Amount operations.
Those operations retain full operand contracts; ordinary Flow × Time remains TotalAmount.
No compatibility production path is retained.

**Implemented:** Replaced scalar material connection families, duplicate receiver
normalizations, generic balance copies, blanket dynamic-without-holdup refusals,
equation-name recycle tears and bespoke vessel closure expressions/checks/reports are
deleted. Unit-owned constitutive laws and scientific accumulated-flux fixture observations
retain their continuing purpose.

The focused verification above establishes these contracts under its stated conditions.
It does not establish whole-product qualification or performance measurements.

### A mistake made and corrected

A derived stock's initial value was initially treated as enough evidence for a composite
inventory's original coordinate initial conditions. That could accept a closed inventory while
the consistent native state violated an authored coordinate value. The implementation now
captures the original initial rows before lowering and assesses their original coordinates
after consistency initialization. A two-coordinate control refuses the inconsistent case
even when its composite inventory closes. Likewise, transfer matching initially used an
authored path string; resolving the actual guard member in the owning instance corrected
nested reuse and preserved source attribution.

The affine boundary control also exposed a reversed representation scale in numerical
projection. Normalization divides a physical residual by its characteristic magnitude,
so the magnitude must multiply by the representation scale. The corrected owner versions
coordinate projection as NumericalProjectionV2 and separately frames admitted difference
projection. Explicit Fahrenheit magnitude/tolerance controls retain policy provenance and
refuse an incorrect point/difference target; historical frame spellings are unchanged.

### Deviations from the plan, deliberate

Only required external prerequisite slices were implemented, as authorized by the maintainer;
the wider D/E/F/H/I plans remain partial. Existing explicit maps and simultaneous initialization
remain their distinct declared realizations. General translators are ordinary authored units,
so physical conversion/reconstruction laws stay with their scientific owner rather than a
second built-in translator language. Full native/Python coupled journeys, integrated checks
and performance measurements remain 25k by the series' execution decision. ADR-0142 and
architecture revision 89 record the new contracts; the ADR remains proposed pending its
decision PR.
