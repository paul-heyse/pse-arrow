---
title: Plan38b common policy and attempt evidence target review
date: 2026-10-13
tier: design
purpose: target
standard: Core 3.4; Heuristics for Efficient Architecture 1.0; Process Simulator 1.5
binding: pse-arrow
decision: Accept
evidence: Interface-checked
disposition_owner: docs/plans/38-efficiency-and-solver-target.md
---

# Plan38b common policy and attempt evidence target review

## Decision and scope

<a id="slot-1"></a><a id="slot-12"></a>

**Accept at the target-document level.** Behavioral/semantic adequacy and architectural
fitness are satisfied for the concrete amendment: proposed ADR-0184, architecture
revision161 additions to blueprint §15.6/§16.5/§18.6/§18.7/§21, and the Plan38b execution
checkpoint. No blocking target finding or additional rule decision was identified.
This accepts the proposed responsibility boundaries and operations; it does not accept an
implementation, close the earlier audit findings, or qualify numerical behavior.

The independent design reviewer examined `main` at
`173ceb2c468a7fe684ec9f0026580186c9c5068c` plus those uncommitted document changes on
2026-10-13. Criteria are Core3.4, its Heuristics for Efficient Architecture1.0 companion,
Process Simulator1.5 and the pse-arrow binding. Current decisions describe the subject;
they do not determine the verdict. The existing audit's pipeline advice supplied source
leads, which were inspected independently where used below.

The target is one physically meaningful request and effective selection/transition meaning
across algebraic roots/NLP, initialization, recycle, fitting/shooting and dynamics, with
truthful bounded evidence and useful failure explanation. Representative workloads are
edit/re-solve, many-case studies, failed recycles, adapter extension and interrupted execution.
The amendment authorizes B1–B5/I-B1–I-B3, minimum C1/C3 native-fact producers and D2 raw-map
samples. Broader native methods, retention/reset, placement, scientific controllers and
storage topology remain separate work. The review does not reopen the full audit or assess
unit/property fidelity, every native status/derivative chain, full-horizon dynamics, capacity
or assembled qualification. No builds, tests, probes, services or measurements ran.

The next consequential step is implementation through the existing owners, preserving the
contracts below. Plan38b owns that work; [Plan38](../../plans/38-efficiency-and-solver-target.md)
owns shared finding disposition and Plan38g owns assembled qualification.

## Governing meaning and owned operations

<a id="slot-2"></a><a id="slot-3"></a>

The proposed seam governs decisions rather than merely standardizing reports. It consumes
an immutable request, adapter-supplied contextual facts, original scientific conclusions,
typed terminal causes, actual bounded observations and remaining aggregate allowance. It
selects the next permitted preparation/execution/assessment or terminal decision. Native
adapters continue to determine actual support/settings/artifact requirements, while the
runtime walker performs effects and retains their full error payloads. Scientific owners
define maps, coordinate transformations, physical merit, branch/start constraints and
original acceptance. These are related responsibilities with distinct reasons to change.

| Responsibility | Proposed owner and contract | Source basis and locality |
|---|---|---|
| Permission, preference and transition meaning | Small pure operations alongside `pse-model::strategy`; injected facts and typed cause projections; no solver/store startup | `strategy.rs` already owns `ProfileRef`, `StartRules`, `BranchPolicy`, `CompositionRequest`, `WorkLimits`, `WorkObservation` and `WorkCharge`. |
| Contextual native support | Native adapter/routing supplies pending evidence, refused, supported-unprepared and ready states, actual settings/representation and immutable build/runtime facts | `routing.rs::Context`, `Decision`, `assess_static` and `Requirements::select_assessed` preserve distinctions beyond static `admit`. |
| Execution and original assessment | Existing runtime walker owns effects, retained proposals, full causes, assessment and aggregate deadline/work ledger | `math/strategy.rs::OriginalConclusion`, `Assessed`, `transition`, `Ledger` and `permits_numerical_continuation` supply existing boundaries to strengthen. |
| Scientific composition | Class/workflow builders supply formulation, derivative and map facts; shared policy consumes their meaning without replacing their controllers | `initialization.rs::PreparedInitialization::strategies`, fitting/shooting curvature resolution and `dynamics.rs::Profile::assess_method` expose real contextual differences. |
| Attempt evidence | Native producers supply actual facts; B owns one compact interpretation consumed synchronously and projected for reporting | `solve.rs::WorkEvidence`, `Evidence` and `SolveReport` already distinguish unknown counters, abandonment, start/reuse and qualification evidence. |
| Ordinary telemetry and recording | Existing bounded optional tap/writer; required terminal recording retains its original fail-closed effect identity | `workflow/durable.rs::Tap::observe` uses nonwaiting queue admission and coverage counters; progress coverage is distinct from scientific completion. |

`pse-model` has no native execution dependency, while `pse-backend-native` already depends on
it. Locating pure policy beside existing meanings therefore admits consumption by native
routing and the runtime without a dependency cycle or native error types leaking into the
model. A typed semantic cause projection is appropriate; transporting full native errors
into this lower layer would change the selected boundary. No universal instruction language,
new planner, crate, trait hierarchy or durable interpreter is needed.

### Physical semantics and well-posedness

| Quantity or model element | Dimension/unit and basis | Convention, validity and authority |
|---|---|---|
| Original candidate and merit | Original physical variable/row units, basis and declared normalization | Scientific owner supplies finite/domain/branch screening and normalized residual/feasibility meaning. Objective alone is not a physical superiority criterion. |
| Raw recycle defect | Each tear's `G(x)-x` in its original compatible coordinates; Err/Tol is dimensionless under the declared physical allowance | D2 owns map evaluation, tear identity, scaling/context and sample role. Accelerated displacement and callback count are different quantities. |
| Native work facts | Counts with method-specific operation meaning; absence is unavailable, distinct from observed zero | Native producer supplies facts; the existing charging owner prevents counting inclusive nested work twice. |
| Phase intervals | Time with declared endpoints and overlap meaning | Actual producer observations govern interpretation; overlapping intervals do not become additive disjoint work. |
| Replay/start/provider inputs | Original physical/provider/reference/coordinate closure | Immutable export retains effective premises and start transforms; missing required premises refuse rather than acquiring present-day defaults. |

Variable roles, formulation and structural witnesses remain with existing model/preparation
owners. Native `routing::Structure` carries original variable/equation identities and a shared
structural-analysis witness; contextual assessment admits that witness before readiness.
Initialization builds block-specific requirements. The amendment neither derives solve order
from stream topology nor interprets absent class evidence as false. Changing representation
or proposing a new point requires its applicable contextual admission and fresh original
assessment; the common policy cannot grant structural, derivative or physical validity itself.
This is preservation of the scoped well-posedness contract, not proof of every analysis mode.

## Revealing scenarios, execution and alternatives

<a id="slot-4"></a><a id="slot-5"></a><a id="slot-8"></a><a id="slot-9"></a>

| Scenario | Expected operation and boundary | Why the amendment fits |
|---|---|---|
| S01 — Add an admitted adapter/profile; binding/mechanism change, PSE-S02/PSE-S06 | Adapter supplies actual contextual facts and method observations; existing policy consumes them | Keeps native API/lifecycle changes local while explicit selection, capability differences and refusal remain visible. |
| S02 — Exercise selection independently; policy change, PSE-S05 | Inject inventories, original permissions, observations and remaining work into pure operations | No native session, canonical store or scientific workflow is required to test branch/start/terminal rules. |
| S03 — Root attempt stops with an auxiliary LS proposal; composition/failure | Independently assess against original roots; screen any allowed transfer; terminal external causes veto successors | Stationarity cannot become root acceptance, and global/multistart/branch permissions are not inferred from eligibility. |
| S04 — Recycle oscillates or a map trial fails; scientific-map/evidence binding | Retain bounded actual raw samples, known/unknown roles, failure and last-successful state | Records what the callback knows without labeling trials as accepted iterations or substituting acceleration norm for physical convergence. |
| S05 — Many cases use advice, then recovery occurs after history changes; policy/dependency change, PSE-S03/PSE-S07 | Freeze effective advice/settings/start dependencies; preserve original occurrence clocks and effects | Avoids a mutable-history lookup during execution/replay and forbids budget renewal through recovery. |
| S06 — Telemetry saturates, writer fails, or cancellation occurs; infrastructure/failure | Decision evidence remains synchronous; ordinary telemetry records loss; automatic summaries use retained evidence | Loss does not change scientific transitions, and explanation does not launch hidden numerical work after a hard stop. |

The current baseline still has independently interpreted choices. In
`math/solves.rs::automatic_operations`, supplied products, causal/reduced/block/direct routes
are composed by statement order, with POUNCE profile recovery before generic alternatives.
`automatic_backend_alternatives` separately reconstructs ranked backend choices;
`routing::Requirements::select_assessed` selects initial class/rank preference. Fitting and
shooting resolve Auto curvature in their own preparation; dynamics supplies its own candidate
assessment. These observations support migrating governing preference, while retaining
scientifically distinct applicability and capability facts. They do not establish that
every current selector is defective or authorize deleting a scientific controller.

| Alternative | Assessment |
|---|---|
| Retain independent selectors and add a common output record | Does not govern request/transition meaning or remove repeated interpretation; inadequate for S01/S02/S03. |
| Put governing policy entirely in native routing | Technically viable dependency direction, but couples scientific branch/start/transition meaning to adapter integration and broadens policy test dependencies. Keep native capability and library-internal Auto there; use the model seam for shared meaning. |
| Selected small pure model operations plus existing walker/builders | Simplest sufficient shared realization. Existing lower-level meanings support it; typed projections preserve runtime causes and adapter facts without moving effects or science. |
| Delegate numerical iteration/globalization/restart/factors to fitting native libraries | Required and complementary. The amendment leaves these algorithms and defaults intact; generic native Auto cannot grant simulator branch, start, global or result permission. |
| Introduce a universal planner, journal or statistical service | No consumed requirement in this amendment earns that machinery. Existing export/recording can support explicit replay; baseline policy operates without history. |

The relevant library comparison concerns ownership of complete numerical capabilities,
not selection of a new solver. `FixedPointOracle::map` already exposes the evaluated `G(x)`;
recording its defect does not require implementing an accelerator or evaluating the map again.
`CausalMap::original_residual` separately evaluates `G(x)-x` and invokes original reconstruction
assessment. The target preserves that distinction and does not change Anderson/damping/restart
defaults. Native APIs/ABI fidelity and supported stop boundaries still require C1/C3's
profile-specific implementation evidence; this review does not assume every library exposes
the same counter, internal slack or interruption capability.

| Numerical stage | Formulation/derivatives/scaling | Class/capability, status and acceptance |
|---|---|---|
| Candidate preparation | Scientific builders retain guard, representation, derivative source/order and normalization requirements; prepare demanded products only | Contextual assessment separates missing evidence/products from incompatibility; explicit options constrain selection. |
| Native attempt | Library owns numerical iteration and factorization; actual profile defines controls and stop fidelity | Typed cause and actual facts enter bounded evidence; policy stop is not result permission. |
| Map evaluation | D2 retains original map and physical tear conversions; raw defect uses the actual callback inputs/outputs | Known roles are preserved and unknown remains unknown; raw history is not an accepted-iterate oracle or a convergence proof. |
| Original assessment and transfer | Independent owner checks new original residuals, bounds, domains, branch and physical allowances; transforms are named and rescreened | Root/LS/VI/global assurances remain distinct; failed/limited proposals cannot acquire use by status or favorable objective. |
| Summary/replay/advice | Retained projections only by default; richer numerical analysis needs admitted work | Missing replay premises refuse; descriptive advice cannot establish causality, eligibility or numerical continuation. |

Execution fit is credible for this added boundary: compact synchronous decisions and bounded
summaries reuse existing facts, and raw-map observation needs no second map solve. Growth in
case/attempt/callback count must encounter finite retained capacity with explicit partiality,
not an unbounded trajectory journal. Optional writer/queue failure does not block numerical
policy. Original aggregate caps and allocation owners survive until actual work drains;
diagnostics cannot silently multiply work after cancellation or exhaustion. These conclusions
apply H3/H4/H6/H17/H24/H26 to this amendment. They neither certify the broader solver route's
scaling nor claim latency/memory improvement.

## Independent architectural and semantic judgments

<a id="slot-6"></a>

Each verdict below assesses the proposed target, with source inspection establishing the
existing seams. It is not a claim that migration or runtime qualification has passed.

| Foundation | Verdict and basis |
|---|---|
| AP-01 | Satisfied: policy, contextual native facts, scientific maps/acceptance and effects have separate owners and variation axes. |
| AP-02 | Satisfied: request permissions, readiness, actual observation meanings, stop fidelity and replay levels are explicit consumed contracts. |
| AP-03 | Satisfied: shared operations govern existing builders/walker; no second orchestration engine or copied numerical algorithm is introduced. |
| AP-04 | Satisfied: original request, branch/start/accuracy constraints, typed causes and original conclusions govern decisions, rather than only report schemas. |
| AP-05 | Satisfied: explicit/native Auto, evidence/artifact/placement states, terminal vetoes and bounded work remain distinguishable. |
| AP-06 | Satisfied: pure policy uses injected semantic facts; native sessions, storage and full scientific workflows remain outside its necessary test setup. |
| AP-07 | Satisfied for the amendment: bounded retained evidence, current callback facts and optional export/telemetry avoid obligatory extra numerical work or durable machinery. |

| Gate | Verdict and evidence |
|---|---|
| G1 | Pass: shared governing operations own policy; adapters/scientific owners supply scoped facts rather than second preference authorities. |
| G2 | Pass: unknown/zero, sample roles, physical merit, native termination/original conclusion and replay levels remain distinct. |
| G3 | Pass: contextual admission, explicit option/profile constraints, transfer screening and missing-premise refusal precede use. |
| G4 | Pass: pure decisions have injected inputs; optional history is frozen; richer diagnostics and effects are explicit. |
| G5 | Pass: hard vetoes, original aggregate allowance, partial telemetry/results and existing recording effect identity govern interruption/recovery. |
| G6 | Pass: original correspondence and checks survive transformations; frozen effective dependencies distinguish rerun from continuation and occurrence recovery. |
| G7 | Pass: unavailable native facts remain absent; actual profile limits qualify stop/counter claims; no convergence or performance benefit is asserted. |
| G8 | Pass: fitting numerical libraries retain iteration, globalization, factors and native state. The shared functions implement simulator permission/composition meaning. |
| G9 | Pass through the seven foundation judgments, limited to this target amendment. |
| PS-G1 | Pass for preservation: original physical/provider/mapping context and physical acceptance remain authoritative. Full model closure/envelope fidelity is unexamined. |
| PS-G2 | Pass for preservation: contextual class/structure admission and original model identities remain required; pending evidence is not a bypass. Full mode-by-mode admission is unexamined. |
| PS-G3 | Pass at target level: original acceptance, derivative/domain obligations, finite declared composition and truthful raw-map history remain required. Native/profile and scientific execution remain unqualified here. |

## Findings, evidence limits and follow-up

<a id="slot-7"></a><a id="slot-10"></a><a id="slot-11"></a>

There are **no new target findings** under the template's falsifiable cause/consequence
standard. The existing audit findings are not resolved by this verdict. Their current
dispositions belong to Plan38; this review adds no competing ledger.

**Rule impacts: none additional.** The amendment implements the already confirmed solver
rule changes linked by Plan38 and does not require a further rule change for the design
recommended here. ADR-0184 remains proposed; acceptance/status changes follow its separate
decision route. No SHOULD exception or MUST waiver is used.

| Claim or risk | Evidence and remaining acceptance boundary |
|---|---|
| Available ownership/dependency seams | **Interface-checked:** model/native manifests, strategy/routing/walker, original work ledger, recycle callbacks, fitting/shooting/dynamics selectors and optional durable tap inspected as described above. |
| One governing policy reaches all consumers | **Proposed:** implementation must migrate actual decision operations, not merely enums/records. Injected policy controls and affected consumer tests belong to B1/B2. |
| Native counters/stops and raw history are truthful | **Proposed:** C1/C3/D2 must establish actual selected-profile meaning, unknown versus zero, sample roles, terminal precedence and raw-map controls. No native test ran in this review. |
| Advice is bounded and honest | **Proposed:** matched complete observations support only declared descriptive comparisons. Partial/censored/unmatched comparisons keep baseline ties and distinct outcome counts; complete matching alone does not prove causal superiority or remove adaptive exposure bias. No automatic numeric tuning is selected. |
| Replay/diagnosis preserve scope and effects | **Proposed:** retained-only hard-stop summaries, immutable reconstruction/effective dependencies, missing-premise refusal, new rerun identity and nonrenewing occurrence recovery need B4/B5 controls. A point vector never proves native continuation. |
| Scientific/native/assembled adequacy | **Not run:** no product build/test/service/probe/measurement. No unit/property model was changed, so no model conformance suite or reference comparison was exercised. Plan38g remains the assembled acceptance owner. |

Preserve the reviewed distinctions during implementation. A concrete inability to project a
required cause/fact without importing native effects, or a selector that must reinterpret the
shared permissions, would reopen the selected seam. A required advice comparison outside
equivalent branch/start/accuracy/assurance scope, or a claimed continuation without real
library state, would reopen its support boundary. Until such evidence appears, the amendment
provides a sufficient target for the authorized bounded implementation.
