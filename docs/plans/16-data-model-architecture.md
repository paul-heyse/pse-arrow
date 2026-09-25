---
title: Data-model architecture and consolidated review remediation
status: in-progress
date: 2026-09-24
adrs: [ADR-0082, ADR-0083, ADR-0084, ADR-0085, ADR-0086, ADR-0088, ADR-0089, ADR-0090, ADR-0091, ADR-0092, ADR-0093]
phase: 1
evidence: Implemented — P00–P09 scoped packets; P10–P18 and full-plan qualification pending
---

# Data-model architecture and consolidated review remediation

## Context

This plan actions the aggregate recommendations from the
[comprehensive codebase review][R1] and the
[data-model architecture follow-up][R2]. `F01`–`F28` refer to the comprehensive
review; `N01`–`N10` refer to the follow-up. Their unnumbered extension, library,
observability and authority recommendations are included below as well.

The selected target is the follow-up's **contract-centered, library-owned
architecture**: authored meaning has one owner, ordinary models are composed from
declarations, and checked transformations produce the representations required by
libraries. The work repairs the original review's concrete defects while amending
remedies that would lose meaning, remove useful behavior or create new duplicated
policy. Reducing independently maintained semantic decisions is the objective;
line count is not an acceptance criterion.

The [charter][CH] supplies the requested architectural intent. Its successor
[layered standard][ST] supplies current DP/PS requirements. Reviews are evidence;
changes to accepted contracts follow the ADR and blueprint routes in P00. This
document sequences those changes and does not itself amend accepted authority.

**Starting point.** Plan 14 M00–M22 is complete for its recorded local Linux
design-stage scope in the [M22 packet][M22]. Its observations remain historical
evidence for the paths exercised. Newly identified defects are forward work here;
this plan neither reopens M22 nor treats its passing cases as proof of these new
paths. The original review's pending-M22/proposed-ADR statements are stale, as the
follow-up explains. Plan 15 remains the separate build-performance workstream.

**Execution status.** P00–P04 are implemented under the scoped
[execution packet](16-p00-p04-execution.md); its verification and supported boundary
are recorded there. P05–P06 are implemented and scoped-qualified in the
[numerics/facts packet](16-p05-p06-execution.md). P07–P09 are implemented and scoped-qualified
in the [native strategies/dynamics packet](16-p07-p09-execution.md). P10–P18 remain pending.
The original reviews provide source
analysis and selected prior observations, not executed acceptance of this target.
A packet must resolve any unverified library claim that matters to its change
before depending on it. If a reported defect is disproved by the current source,
record the concrete reason in its coverage row; do not implement a speculative fix
merely to match the review. No implementation, test campaign or environment repair
was part of initially creating this plan.

## Target and completion boundary

The completed design supports these public Rust and Python journeys:

1. Declare a reusable unit/property model with typed ports, valid index domains,
   equation/balance templates and formulation choices; instantiate and bind it
   without another handwritten copy of its physics.
2. Load or build a case whose requested declarations are consumed, explicitly
   retained as nonexecuting data, or refused with source-attributed diagnostics.
   Unsupported meaning never silently disappears.
3. Bind species to actual parameter records, enforce quantity kinds and reference
   conventions, and execute a conserved reaction example through declared equations
   and contributions. Missing required physics is refused before solving.
4. Resolve accuracy, scaling, derivative requirements, bounds and capabilities
   before selecting a native route. Square roots, declared fixed-point maps,
   NLP, LP/MILP, admitted convex QP and explicit conic models have truthful routes.
5. Execute a recycle and staged initialization with explicit strategy, starts,
   overrides and history; a failed stage leaves the original specification intact.
6. Reuse the same model definitions for steady solving, dynamics and fitting where
   their declared formulations apply. Time origins, event/reset semantics,
   sensitivity validity and incomplete trajectories remain explicit.
7. Return typed candidates, qualification, physical checks and diagnostics through
   Rust, Arrow, Python and storage without reconstructing their meaning in each
   adapter. A useful best iterate or derivative remains available with its limits.
8. Rebuild cases and run studies using complete reuse dependencies, generous finite
   resource policy and retained Rust build caches; record the starts actually used.
9. Publish repeatedly to a logical base, distinguish conflict from uncertainty,
   settle an interrupted attempt and reopen exact results on read-only storage
   under an explicit compatibility and retention contract.
10. Extend an ordinary model by declarations and bindings; add new implementation
    only for genuinely new physical or numerical operations behind checked
    contracts. Inspect resolved decisions and original-model outcomes.

Qualification is local Linux using the pinned default and native feature profiles.
Distributed execution, GPU support, arbitrary global MINLP, general higher-index
DAEs, release certification and wheel/platform campaigns are not added. These
boundaries do not authorize silent fallback or broader capability claims. No new
general expression engine, universal workflow language, rule engine, mega-registry
or design-alignment framework is part of the plan. Library eligibility remains
open under ADR-0066; fit to an actual operation determines selection.

## Consolidated design decisions

These are **Proposed** decisions to implement through the existing owners. Names
describe responsibilities, not a mandate for a new crate or struct for every row.

| ID | Selected direction | Amendment to the review proposals |
|---|---|---|
| T01 | Separate model definitions, case bindings, analysis requests, resolved numerical policy, prepared products, attempts/results and publications | Strengthen existing `ModelRevision`, `Inputs`, `CasePlan`, profiles and reports; do not add a parallel semantic model |
| T02 | Exhaustive semantic admission within the selected model, dependencies and analysis | Refuse unsupported requested meaning; do not reject unrelated stored documents or delete required-but-unwired contracts to make admission appear complete |
| T03 | A provider binding includes actual species/parameter correspondence, port kinds, basis, reference, phase/formulation and validity | A kind string or closed enum plus an arity check is insufficient; use typed registered bindings without freezing library eligibility |
| T04 | Resolve numerical policy once, then derive library options and original-space acceptance criteria | Neither the raw maximum nor the minimum of differently dimensioned tolerances is valid; physical closure and solver feasibility remain distinct observations under an explicit acceptance policy |
| T05 | Distinguish requested, available, admitted, eligible and selected capabilities | One owner supplies static adapter inventory and contextual eligibility; neither is automatically a promise that the public workflow supports every class |
| T06 | Derive execution strategy from model relationships, admitted equations and analysis policy | Topology SCCs alone do not select KINSOL; support explicit causal fixed-point, root, simultaneous constrained and continuation strategies through existing libraries |
| T07 | Separate native termination, candidate kind, feasibility, stationarity, optimum/gap, completeness and derivative/identifiability conditions | An acceptable stop is not automatically stationary; a rank-deficient fit can still provide a valid response Jacobian at its candidate |
| T08 | Define versioned identity projections for each semantic and reuse scope, then share canonical framing | Do not serialize whole objects indiscriminately. Preserve the existing signed-zero contract. A source hash establishes input identity, not equivalence or evidence applicability |
| T09 | Use isolated immutable member paths for result publications, an explicit parent precondition and attempt settlement | Loading an existing table alone does not repair the protocol; shared-table overwrite needs its own demonstrated operation and contract |
| T10 | Consolidate mechanics only when validity and lifecycle semantics coincide | Content-complete cache keys do not make clear/eviction fences unnecessary; program validity, retained memory and storage snapshot invalidation differ |
| T11 | Delete replaced or unnecessary bespoke mechanisms after their required behavior has a surviving owner | No blanket deletion of authoring, inspection, retention, generated typing or useful library capability; no line-count target for codegen or tooling |
| T12 | Keep factual execution records and ordinary test tooling; use review documents for design judgment | Remove compulsory digest-bound review verdicts and plan-specific phase ceremony. Label executed, unchanged-input reuse, reviewed transfer and not-run evidence distinctly |
| T13 | Declare determinism, environmental effects, phase policy, trial recovery and approximation | Fixed symbol registration is a control, not proof of bitwise reproducibility. Mechanical stability is not global phase stability; smoothing and numerical PSD evidence change assumptions that must be recorded |
| T14 | Keep resource controls configurable and finite for the 192 GB, 16-core/32-thread workstation | Repair bad estimates rather than tightening arbitrary small limits. Charge retained, active and foreign memory separately; observed RSS is a measurement, not the reservation ledger |

### Ownership and representations

| Meaning | Existing primary owner to extend | Derived consumers and enforcement |
|---|---|---|
| Reusable definitions, domains, relations and physical conventions | `pse-schema`, `pse-authoring`, `pse-quantity`, `pse-material` | Registry/native types share one declaration per meaning; builders and document loading enter the same semantic admission |
| Selected immutable model and case/analysis binding | `pse-runtime::workflow`, compiler `Inputs`/`CasePlan` | Validate selected dependencies, publish one coherent revision, preserve template/instance/source mappings |
| Typed mathematics, coefficient facts and structural projections | `pse-compiler`, `pse-math`, `pse-structural` | Symbolica/Numerica, pounce-presolve and graph libraries own algorithms; typed mappings preserve domain identity |
| Material/provider contract and implementation data | `pse-kernels` registration plus authored material data | FeOS/num-dual workers use an admitted coordinate/data binding; runtime does not maintain another material registry |
| Numerical and execution policy | Existing native profiles and workflow preparation | Resolve model attributes, explicit request and selected defaults once; adapters consume the effective contract |
| Mutable native work and result observations | Existing runtime jobs/sequences and native adapters | Attempt owns mutable state and resources through join; shared model data stays immutable |
| Result vocabulary and diagnostics | Existing report/error owners, schema projections and diagnostics crate | Rust, Arrow and Python expose the same structured meaning and source references |
| Compatibility and publication | `pse-schema` resolved contracts and `pse-catalog` | Complete semantic contract closure, explicit encoding identity, exact committed members and parent/attempt protocol |

The owner of each invariant must be identified in the implementing change. A second
editable policy or vocabulary is not justified by placing it in a different layer.
Native numerical iteration stays library-owned; Arrow/DataFusion remain appropriate
for admission, set-oriented model/result work, inspection and storage boundaries.

## Execution method

Use the existing checkout and stable Cargo target/cache locations. Preserve unrelated
dirty work. Implement one dependency-ready packet at a time; no continuously running
review agent is needed. Final independent review occurs in P18.

Each packet compiles touched packages with `just check-package <pkg>` (or `just
check` for cross-crate changes) and runs targeted behavior tests through `just
unit-package <pkg> <filter>`. Test names below are proposed semantic cases, not claims
that these tests already exist. Use the recipe-owned pinned toolchain and explicit
force-validation feature. A new mechanism receives its targeted tests in the same
change. Use focused boundary/library characterization when an uncertainty actually
affects the implementation; do not turn every packet into a full acceptance run.

Run `just codegen` whenever a registry or generator changes. Edit generators, never
generated output. Run `just family-check` when pinned-family dependencies change.
Delete a displaced mechanism, its callers and obsolete tests/fixtures as soon as
the replacement is proven. P16 reconciles remaining candidates; it is not a reason
to defer deletions from earlier packets. Do not add compatibility execution paths.

One final stage, P18, owns the complete integration, component, native solver,
Python, performance and repository qualification. After a failure, repair its owner
and rerun affected checks; do not repeat unrelated campaigns after documentation
edits. Checkpoints record state, decisions and next dependencies, not per-command
receipts or a new numbered evidence system.

### Packet dependency table

All states start **pending**. Packet order is a dependency order, not a demand for
parallel agents. P17 can run immediately after P00 to simplify the execution tools.

| Packet | Deliverable | Dependencies | Main review coverage |
|---|---|---|---|
| P00 | Target contract decisions, authority routes and bounded library checks | — | N01–N10; F27; review authority tables |
| P01 | Literal admission and safe, explicit symbolic runtime entry | P00 | F03/F04 |
| P02 | Shared semantic vocabulary, identity projections and diagnostic foundations | P00 | F06/F10–F12/F16/F18/F24; N03/N05–N07 |
| P03 | Complete selected-model admission and declarative composition | P01/P02 | F01/F02/F24; N01/N04; ordinary extension |
| P04 | Material/reaction and physical provider binding | P03 | F02/F08/F10; N02 |
| P05 | Model-owned numerical policy and normalized acceptance | P02/P04 | F05/F07/F09; N03 |
| P06 | Shared mathematical, coefficient and structural facts | P01/P03/P05 | F01/F16–F18/F24; N03/N07 |
| P07 | Actual native routes, contextual capabilities and qualification | P05/P06 | F01/F05/F18; N03/N06 |
| P08 | Starts, recycles and initialization through immutable case overlays | P03/P06/P07 | F01/F13/F15; N04/N05 |
| P09 | Coherent dynamics, event policy and fitting | P04/P05/P07/P08 | F10/F18/F20; N02/N04/N06 |
| P10 | Typed result/diagnostic round trips and observable execution | P07/P08/P09 | F05/F06/F13/F16/F24; N06/N07 |
| P11 | Correct reuse, cache lifetimes and prepared-data ownership | P02/P06/P08/P09 | F11/F13/F23/F28; N05/N09 |
| P12 | Semantic compatibility and recoverable immutable publication | P02/P10 | F12/F14; N05/N08 |
| P13 | Generated and consumer-contract consolidation | P03/P06/P10/P12 | F22/F24; N01/N05/N09 |
| P14 | Proportionate resource admission and sparse fitting | P06/P09/P11 | F19; N09 |
| P15 | Targeted repeated-case cost improvements | P11/P14 | F28; selected ledger substitutions |
| P16 | Resolve remaining surplus and dependency directions | P03–P15 | F21–F24/F27; N09 |
| P17 | Ordinary execution tooling and honest evidence | P00 | F25/F26; N10; tooling ledger |
| P18 | One final qualification, independent review and plan closure | P01–P17 | All findings and all current DP/PS gates |

## Implementation packets

### P00 — Resolve target contracts and authority changes

**Responsibility:** decision documents, the live architecture description and the
contract boundaries named above. Do not begin by rewriting every historical plan.

- Convert T01–T14 into concrete owner/update/enforcement decisions at the relevant
  existing types. Resolve which registry or Rust declaration owns each vocabulary;
  decide identity equivalence scopes before changing hashes.
- Prepare the required ADRs before affected implementation: model/provider and
  numerical contracts; identity/compatibility; Python/result/error boundaries;
  publication/retention; and governance/tooling. Group genuinely related decisions;
  do not create an ADR for each local bug. Assign new numbers through repository
  tooling, not placeholders presented as accepted records.
- Supersede accepted arguments where necessary. ADR-0082–0087 are not editable
  proposed drafts. Preserve ADR-0087/M22 historical scope. Correct active blueprint
  catalogs, D12/current architecture text and consumer claims through a `design:`
  change with a revision row. Historical rationale can remain clearly marked or
  be archived with links; deleting every old name is not the acceptance test.
- For uncertain library proposals, use the matching local library skill, pinned
  source/capability evidence and current documentation via Context7 where needed.
  Keep an API recommendation provisional until its actual contract fits the
  operation. Required decisions are assigned below; there is no open-ended library
  research phase or new capability registry.

**Targeted completion:** each contract-changing packet names its approved decision
and existing implementation owner; the two reviews' conflicting prescriptions have
one selected disposition. Applicable ADR review requirements are met before those
changes, without imposing approval on ordinary bug fixes.

**Remove/correct:** contradictory active consumer/authority claims. Preserve dated
observations and accepted ADR bodies except for permitted supersession metadata.

### P01 — Repair authored powers and make symbolic runtime entry safe

**Responsibility:** `pse-math::{typed,library,lib}`, compiler typed admission and the
runtime's symbolic job boundary; F03/F04.

- Preserve exact literal/rational facts when constructing exponent atoms; check
  numerical agreement without comparing incompatible float and rational encodings.
  Keep dimensioned-power and domain obligations intact.
- Use the existing personal Symbolica license configuration. The P00–P04 scope
  explicitly excludes additional license management, a restricted execution mode
  and process isolation. Initialize the symbolic context before tracked work and
  never persist a license secret. Library eligibility is unchanged.
- Move initialization and symbol/function registration out of apparently pure
  tracked derivations. Resolve stable names/registration and effective environment
  dependencies. Specify the reproducibility class by operation and build/profile;
  do not promise whole-program bitwise identity from registration order alone.
- Capture actual linked native-library versions through available library APIs
  where results depend on them, rather than assuming a declared version string
  proves what the loader used. Reuse this environment identity in P02/P10/P11.

**Targeted checks:** authored `x^2`, `x^-1` and normalized-temperature cubics through
public compiler admission, with analytic values/derivatives; concurrent initialized
jobs and reversed model admission order across fresh processes under the declared
determinism profile. No absent/restricted-license behavior is claimed.

**Delete:** the mismatched exact-literal comparison and hidden initialization path
when the corrected entry boundary is in use.

### P02 — Establish shared vocabulary, identities and error structure

**Responsibility:** existing domain/report/profile enums, `pse-ids`, schema
declarations, diagnostics and the Rust/Python boundary.

- Define stable tags for backend, termination, candidate kind, qualification,
  missing/unavailable evidence and metric alternatives. Declare heat-in/heat-out
  roles separately from work and define time origin versus elapsed duration.
  Render/parse Rust, Arrow and Python spellings from one owner.
- Define semantic projections for definition/body content, instance/revision, case,
  analysis policy, specialization, prepared artifact, native layout, used start,
  result and publication. Decide which fields affect each scope and version the
  algorithms. Keep semantic schema identity separate from documentation and
  DataFusion/native encoding identity.
- Centralize framed floating-point encoding and matching semantic equality.
  Preserve signed zero as the existing contract requires; canonicalize NaNs only
  according to the declared admissible-value contract. Reconcile diagnostic-only
  definition IDs with reusable body-content identity.
- Introduce the domain distinctions needed for invalid model, unsupported
  capability, resource exhaustion, recoverable trial, nonfinite evaluation,
  native infrastructure, cancellation, conflict, incompatibility and internal
  failure. Keep codes, stage, source/instance references and typed observations;
  message formatting belongs to presentations.
- Specify one capability record with explicit static and contextual scopes,
  including bounds, admitted derivatives, starts, threading and problem classes.
  The selection algorithm is implemented in P07, not independently in Python.

**Targeted checks:** vocabulary encode/decode and capability-to-settings round
trips; distinct failure classes; signed-zero/NaN contract cases; controlled edits
to identity projections. State expected invalidation and non-invalidation, not
“every field must change every hash.”

**Delete:** duplicate spellings/sentinels and ad hoc float framing as callers move.
Use existing type owners/re-exports to avoid introducing crate cycles.

### P03 — Make admission complete and ordinary composition declarative

**Responsibility:** `pse-runtime/src/workflow/{model,sources}`, schema document
contracts, authoring/builders and compiler `Inputs`.

- Compute the selected model/dependency/analysis scope. Exhaustively classify its
  declarations as executable, explicitly nonexecuting or unsupported, with a
  source-attributed refusal. A retained document is not advertised as consumed.
  Both document loading and typed builders use this boundary.
- Complete the reusable unit/template contract: typed parameters, valid index
  domains, variable/equation templates, instance bindings, port roles,
  contributions, formulation guards and supported modes. Preserve compact
  templates until specialization and retain template/instance/source mappings.
- Bind connections and material/property/scaling/tear declarations into compiler
  inputs. Populate the real flowsheet projection from ports/connections; prohibit
  extensive-flow fan-out unless an explicit splitter/conservation formulation
  accounts for it. Keep information and physical connections distinct.
- Make compound model edits publish a checked immutable revision. Flow and
  initialization preparation receive the selected revision explicitly rather
  than consulting whichever inputs were last published.
- Turn specialized convenience constructors, including the vessel, into producers
  of inspectable declarations where they encode ordinary model choices. Do not
  demand that a genuinely new numerical/physical algorithm be represented as
  arbitrary configuration.

**Targeted checks:** selected reaction/scaling/tear data is used or refused; an
unrelated stored case can remain nonexecuting; equivalent builder/document input
produces the same admitted meaning; reordered declarations preserve identities;
invalid connection multiplicity and cross-revision preparation fail early. Add an
ordinary heater/unit variant without another capability or equation vocabulary.

**Delete:** silent fall-through and alternate entrypoints that bypass semantic
admission; replace duplicated constructor physics once authored templates serve it.

### P04 — Bind materials, reactions and physical property semantics

**Responsibility:** `pse-material`, `pse-quantity`, `pse-kernels::feos`, provider
registration, authored property/material rows and workflow bindings.

- Bind declared species identities to actual FeOS parameter records and ordered
  composition coordinates. Validate duplicates, missing species, dataset identity
  and permutation mappings; a label/arity match is not a data binding. Support the
  admitted dataset's shape rather than disguising a fixed three-species package
  as a generic provider.
- Validate complete port quantity kinds, dimensions, basis, reference and phase
  semantics before constructing a worker. Separate enthalpy and entropy reference
  conventions, including pressure where required. Resolve the entropy-offset
  claim against the pinned implementation and an independent reference before
  choosing an explicit conversion; no unexplained constant correction.
- Declare homogeneous/phase-equilibrium formulation, validity envelope,
  stability requirement and permitted extrapolation/trial behavior. A mechanical
  derivative check and a global phase-stability diagnostic remain different
  evidence. No unconditional phase rejection silently changes the selected model.
- Consume material-system, reaction and stoichiometry declarations for the
  supported reaction model: validate species/element closure, bind authored
  reaction equations/rates and generate conserved contributions. Energy-producing
  reaction use requires declared formation/reference data or an explicit heat
  model. Refuse missing required physics rather than inventing it.
- Correct vessel heat roles through the shared contract. Record data provenance
  and the selected provider/phase/reference policy in the resolved model and result.

**Targeted checks:** species permutation maps correctly or is rejected, unknown
species and H/U kind swaps fail, entropy agrees under a declared reference,
stability policies distinguish admissible states, an unbalanced reaction is
refused, and a balanced executable reaction closes material/element/energy accounts
under its declared formulation.

**Delete:** unchecked bundled-data labels, arity-only acceptance and duplicated
reference interpretation. Keep the bundled dataset as an ordinary valid binding
if useful; do not duplicate FeOS thermodynamics.

### P05 — Resolve scaling, tolerances and physical acceptance once

**Responsibility:** authored scaling/nominals, native profiles, presolve mappings,
workflow preparation, closure and original-space quality checks.

- Resolve precedence explicitly: model nominal/scaling declarations, explicit
  case/analysis choices and selected defaults. Separate model normalization from
  algorithmic scaling. Validate signs, dimensions and completeness.
- Represent variable, row and observable absolute/relative tolerances with their
  physical or normalized coordinates; give KKT, integrality and MIP gaps their
  own meanings. Carry the corresponding derivative/dual transformations.
- Feed presolve a normalized contract whose tolerances are comparable. Preserve
  original-space certificates/witnesses and round-trip transforms. Neither raw
  `max` nor raw `min` across unlike units is a permitted shortcut.
- Derive Ipopt/POUNCE scaling and acceptable/bound options, KINSOL variable/residual
  scales and strict-sign constraints, and HiGHS/Clarabel controls from this contract.
  A recorded library scaling fallback is allowed by explicit policy; it must not
  override an authored requirement invisibly. Prevent relaxed bounds from
  violating original guards or misrepresenting final feasibility.
- Resolve solve, integrated-output and conservation tolerances as related but
  distinct requirements. Report achieved residuals and each acceptance decision;
  a final policy explains whether a physically unclosed candidate is usable.

**Targeted checks:** unit-rescaled equivalent cases have equivalent acceptance;
mixed energy/composition rows cannot erase infeasibility; scale/dual recovery;
guard-bound behavior; policy precedence; deliberately distinct numerical and
closure criteria are reported without a false success claim.

**Delete:** scattered fallback interpretation, ignored authored scaling and the
mixed-unit scalar presolve tolerance. Regenerate changed profile/source contracts.

### P06 — Consume single mathematical and structural facts

**Responsibility:** `pse-math::{facts,coefficients,presolve}`, compiler projections,
`pse-structural` and native presolve/postsolve.

- Derive admitted derivative order, bound shape, guarded domain and class facts
  from the admitted program, not requested settings. Keep value-dependent facts
  tied to the exact coefficient/value inputs that establish them.
- Correct transcendental interval/FBBT projection using actual pinned Symbolica
  symbols and normalized forms. Share coefficient/affine facts and discharged
  obligations across presolve and coefficient solvers where the contracts coincide;
  do not classify the same guarded affine row differently by backend accident.
- Preserve structural block identities and declared order, including algebraic
  partition scope. A different valid topological order is not inherently wrong;
  positional interpretation without an identity mapping is forbidden.
- Preserve presolve infeasibility proofs and source row/column/contribution
  mappings. Name cycle edges and forbidden-tear conflicts. Add finer structural
  witnesses through library graph operations only where coarse witnesses cannot
  answer the required diagnostic.
- Produce convex-QP evidence through an admitted coefficient path. Retain exact
  Gram certificates; represent numerical PSD assessment separately with snapshot,
  tolerance and inconclusive state. Never repair an indefinite matrix silently.

**Targeted checks:** log/exp and other supported normalized forms participate in
FBBT; guarded literal division has one affine classification; reordered blocks
retain identity; deficient systems and infeasible rows name their sources;
exact/numerical/indefinite/inconclusive quadratic cases are distinguished.

**Delete:** competing affine classifiers and block-order reconstructions that add
no new semantics. Preserve required source and postsolve mappings.

### P07 — Make capabilities executable and native outcomes defensible

**Responsibility:** native routing/adapters and shared workflow preparation,
including coefficient and explicit-conic request paths.

- Derive requirements and eligible alternatives from P02/P05/P06. Separate static
  linked adapter inventory from usable public workflow support. Reject unsupported
  nonsmooth, bound, derivative and thread combinations during preparation.
- Complete public routes for square equations, declared causal fixed-point maps,
  NLP, LP/MILP, convex QP and explicitly represented cones. Replace the caller's
  coefficient-selection boolean with derived preparation or an explicit typed
  analysis request. Do not infer arbitrary nonlinear constraints as cones.
- Select constrained alternatives when KINSOL cannot satisfy the admitted bounds;
  do not silently drop bounds or reinterpret an arbitrary residual as a causal map.
  Propagate effective thread policy through all paths, including POUNCE fits.
- Map actual native status constants and linked versions into typed termination.
  Grant stationarity/optimality only from the relevant checked evidence and
  declared policy; reserve and record acceptable-level, gap and bound options.
  Distinguish resource exhaustion, inconclusive stop and ordinary iteration limit.
- Check coefficient-class candidates against original model values, independently
  of the solver upload transformation. Keep exact optimum, gap-qualified optimum,
  feasible point and best iterate separate.

**Targeted checks:** every advertised public class has an admitted route; boxed
roots and nonsmooth requests select a valid alternative or fail before worker
acquisition; status tables, loose acceptable-level stops, MIP gaps, memory/unknown
codes and an intentionally defective upload cannot overclaim success.

**Delete:** duplicate eligibility tables and misleading capability rows, late
contract refusals that preparation can decide, and universal `successful()` logic
that discards the required qualification distinctions.

### P08 — Wire explicit starts, recycles and staged initialization

**Responsibility:** workflow strategy selection and existing structural, tear,
initialization, sequence and native start owners.

- Separate immutable case bindings, temporary stage overlays, solved unknowns and
  the original user specification. A failed or cancelled stage retains the active
  overlay as evidence and cannot return it as the authoritative specification.
- Separate native allocation/model reuse from numerical start policy. Record the
  actual primal/dual/basis seed, origin attempt/result, selected parts, coordinate
  compatibility and transformations; output seed availability is a different fact.
  Clear/reset retained native starts when policy disallows them, including HiGHS
  basis state. Record warm-start bound-push choices.
- Consume authored tear costs/preferences and the selected revision's flow graph.
  Respect forbidden edges in exact and heuristic selection; report a concrete
  cycle when the allowed policy cannot break it. Preserve the exact tear MILP and
  independent acyclic witness.
- Derive an inspectable strategy from distinct topology/incidence projections.
  Support explicit causal fixed-point/Anderson blocks, root blocks, simultaneous
  constrained blocks and continuation where admitted. Libraries own iteration;
  strategy data selects it. Record iteration/stage residuals, limits and failures
  without inventing metrics an adapter does not expose.

**Targeted checks:** temporary override failure leaves the original case intact;
multi-root sequence versus standalone solve is explained by recorded seed use;
Fresh/no-start behavior agrees; forbidden tear and extensive fan-out cases fail
correctly; a recycle converges through the selected tear route with interpretable
history and can fall back only under an explicit eligible strategy policy.

**Delete:** implicit reuse-implies-warm-start behavior, last-published preparation
lookups and merged specification/result maps.

### P09 — Complete shared time, dynamic and fitting contracts

**Responsibility:** native dynamics/integrator, workflow dynamics/vessel/fitting,
event contracts and the fit oracle.

- Convert observation time coordinates once into the integration coordinate using
  the declared origin and units. Apply it consistently to sampling, events,
  predictions and result rows; do not require `start == 0` as the permanent fix.
- Represent formulation/domain and trial-failure policy explicitly. Distinguish a
  recoverable trial from a terminal model/infrastructure failure where the pinned
  integrator supports recovery. If the chosen method cannot meet a required
  operation, qualify an appropriate library route (including IDA/IDAS if needed)
  rather than adding a bespoke step-retry loop.
- Give the valve/back-pressure behavior a declared physical mode: directional
  closure/reversal/event or explicitly parameterized smoothing, as appropriate to
  the model. Include selected approximation in identity and physical checks.
- Complete same-layout event/reset sensitivity support only after checking event
  timing dependence, reset derivatives, consistent initialization and the library's
  actual sensitivity contract. Distinguish scheduled events from state-triggered
  events. Refuse unsupported grazing/nonsmooth/layout-changing cases truthfully;
  do not infer full hybrid-fit support from one reset method's existence.
- Make smooth and admitted evented fits consume shared model/physical/numerical
  contracts and thread policy. Keep response derivatives at a candidate distinct
  from estimator sensitivity, rank, conditioning and uncertainty.

**Targeted checks:** nonzero-origin predictions and known-parameter recovery;
conserved blowdown to back-pressure under its declared formulation; recoverable
trial behavior or preparation refusal; scheduled and state-triggered reset
derivatives against independent piecewise analytic cases; a supported evented fit;
typed refusal of unsupported hybrid sensitivity. Full solver journeys run in P18.

**Delete:** duplicated absolute/elapsed-time interpretation, blanket event refusal
for newly supported cases, and hidden valve/domain assumptions. Keep explicit
refusals for behavior the completed contract does not admit.

### P10 — Preserve result meaning and diagnostics through every view

**Responsibility:** native reports, workflow result encoders, schema result rows,
`pse-py`, Python wrappers/stubs and existing execution observations.

- Populate P02's typed outcome contract with native termination, candidate kind,
  primal/dual/KKT/gap observations, policy-qualified assurance, physical closure,
  completed trajectory prefix and absent/failed evidence states.
- For fitting, publish values as a qualified estimate or a labeled candidate;
  expose feasibility, convergence, rank and conditioning as typed data. Keep valid
  response Jacobians at candidates with their validity conditions. Do not report
  covariance or estimator sensitivity unsupported by identifiability evidence.
- Carry model/case/request/preparation IDs, actual start/environment identities,
  parameter-data provenance and effective options into the result's lineage.
  Result immutability and encode-once ownership remain intact.
- Map diagnostics to source spans, unit/port/instance names and structural or
  presolve witnesses. Preserve provider/demand attribution, avoid attaching a
  callback fault to an arbitrary first variable, and surface final fit errors in
  Python. Use library parsing spans/path errors and miette/Python presentation
  over the same classification, with no second taxonomy.
- Expose bounded existing tracing and completed-plan metrics for slow/failed
  inspection and publication. Distinguish unavailable metrics from zero and do
  not reevaluate work to observe it. Remove lineage constants that claim a policy
  which no implementation enforces.

**Targeted checks:** result round trips across Rust/Arrow/Python for iteration
limit, acceptable, rank-deficient, incomplete and failed cases; diagnostic source
mapping; capability-to-settings spellings; no `Debug` or sentinel is the durable
semantic contract; enabling observation preserves outcomes and ownership.

**Delete:** `Debug`-encoded durable vocabulary, hand-built Python capability rows,
hidden fit diagnostics and fictitious lineage fields.

### P11 — Unify reuse semantics without conflating lifetimes

**Responsibility:** compiler input equality/identity, runtime artifacts and flights,
native retained sessions, Salsa lifecycle and existing storage caches.

- Apply P02's identity projections to all reviewed omissions: dynamic output
  tolerances, effective solver/start settings, HiGHS sparse starts, KINSOL scales,
  provider data/environment and relevant value-dependent assumptions. Reconfigure
  or rebuild retained native state when these change; layout equality alone is
  insufficient. Validate clean versus reused preparation under the same semantics.
- Share immutable definitions, bodies and layouts through existing ownership;
  attach case/attempt ownership without deep-cloning symbolic programs. Permit
  dynamic parameter/horizon rebinding only through checked dependencies that
  invalidate affected modes, samples, facts or preparation.
- Separate compiler dependency reuse, compiled-program storage, storage snapshots
  and mutable native state. Remove storage-maintenance invalidation of unrelated
  math artifacts; retain an explicit program-cache clear/eviction policy and its
  late-insertion fence.
- Consolidate repeated fenced-cache mechanics only where charge, insertion,
  eviction and invalidation contracts coincide. Retiring flights receive a
  distinct retryable state; cancellation never becomes a memoized result. Reuse
  the adopted cancellation primitive where its semantics fit and retain native
  leases through actual worker completion.
- Place shared flight/cache mechanics in the lowest suitable existing owner
  without creating a dependency cycle; a new crate requires P00's ADR route.
  Assign Salsa durability by semantic stability and replace arbitrary publish/
  prepare-count rotation with an evidenced interned-resource bound. Preserve
  cancellation, membership and absence tracking through rotation.

**Targeted checks:** signed-zero and each relevant policy edit versus clean
preparation; KINSOL tolerance change; clear during in-flight build; cancelled then
fresh request; storage maintenance retaining valid programs; bounded Salsa growth;
correct rebind invalidation. Stress lifecycle behavior without using wall time as
the correctness oracle.

**Delete:** duplicate cache wrappers whose contracts now match, raw hash/equality
paths, unrelated invalidation hooks and unnecessary cross-layer imports. Preserve
fences that enforce retention rather than computational validity.

### P12 — Complete semantic compatibility and publication recovery

**Responsibility:** schema fingerprints/resolved contracts and catalog/workflow
publication, settlement, exact reads and retention.

- Define a semantic contract digest over the complete required relation, field,
  enum, extension, check and integrity closure. Exclude prose and incidental
  library encodings from semantic identity; retain their versions separately.
  Persist check meaning in the admitted canonical contract, not solely opaque
  DataFusion protocol bytes. Bind compatibility to the compiled consumer.
- Define admission of older recorded contracts: compatible interpretation under
  the recorded version or explicit typed incompatibility/migration requirement.
  Do not build a general historical runtime or silently treat equal hashes as a
  migration. A documentation-only or equivalent encoding change remains readable.
- Select immutable per-publication member locations for run artifacts. Define
  logical publication, attempt/retry and parent/precondition identities, then
  commit the exact control manifest last. Separate materialization and settlement;
  a failed attempt must not poison the logical base.
- Classify proved conflict/noncommit separately from uncertain commit. Expose
  settle-by-attempt/publication using existing catalog transaction witnesses;
  repeated settlement returns the same established outcome. Do not blindly retry
  uncertain effects or expose partial member output as a committed result.
- Make exact read-only reopen genuinely read-only. Specify reader ownership and
  retention protection independently from a first-read lock-file write. Keep
  supported snapshots protected; retention/GC is an explicit writer operation,
  and cannot claim safety for external readers without a declared protocol.
- Where Delta actions/commit reads are custom, use the pinned library log APIs
  when they preserve size admission, transaction evidence and error semantics.

**Targeted checks:** same-base parent chain, competing children, failed member
write, ambiguous control commit then settle, idempotent settlement, old exact
result access, read-only reopen, semantic versus prose/encoding changes, malformed
contract and active-reader retention. Include a complete artifact containing a
failed numerical attempt to distinguish publication from scientific success.

**Delete:** fixed-path accidental overwrite assumptions, prose-sensitive reopen,
unclassifiable publication errors and read-side mutations replaced by the selected
protocol. Preserve control-last and exact-version guarantees.

### P13 — Consolidate generated contracts and consumer interpretations

**Responsibility:** schema/codegen, relation validation, native enum adapters,
authoring target paths, engine traversal and invariant fixture discovery.

- Replace redundant frozen structural copies only after P12's complete semantic
  contract and independently bound consumer expectations provide the needed check.
  Keep typed generated accessors where runtime reflection cannot serve the same
  compile-time contract. State that reason at the generator.
- Re-export existing authoritative Rust enums where appropriate; use qualified
  enum derives for mechanical conversions. Registry-owned meanings remain
  registry-owned. Remove quantity-enum string round trips without creating a
  second declaration or a schema/domain dependency cycle.
- Compile PK/FK/check obligations once, including nested and composite cases, and
  derive publication and invariant-query forms from that product. Preserve
  differences that represent genuinely different boundary obligations.
- Reuse the DSL path/identifier grammar for targets and edits, and existing DAG
  traversal for matching traversal purposes with shared-subtree semantics. Do
  not unify walkers whose traversal contract is different merely by name.
- Replace generated fixture wrappers with standard runtime discovery only after
  proving stable per-case names, nextest discovery, filtering and failure/empty
  selection behavior. Use typed DataFusion settings such as spill compression
  where they replace a string interpretation.
- Remove incidental module edges, including authoring's math dependency for a
  shared enum, by placing the meaning with its correct existing owner.

**Targeted checks:** generated/compiled contract mismatch, nested/composite FK
agreement, physical enum round trips, Unicode/escaped target paths, shared-DAG
visits, and fixture discovery with intentionally failing/empty cases. Regenerate
through `just codegen`; final equality is checked in P18.

**Delete:** replaced frozen copies, enum implementations, fixture wrappers,
obligation compilers, parsers and traversal copies with their obsolete tests.
Acceptance is preserved contracts and simpler ownership, not generated line count.

### P14 — Make resource admission track real work

**Responsibility:** math construction/execution limits, runtime `MathPolicy`,
prepared/active leases, artifact accounting and fitting sparse assembly.

- Resolve actual Numerica block-local expansion and library allocation behavior;
  replace the unsupported width-squared estimate with a sound bound or bounded
  post-transformation admission. Do not substitute another unproved magic factor.
  Preserve exact derivative semantics at larger coordinate counts.
- Prevent exponential `let` expansion by retaining shared/block-local structure
  and checking growth during construction, before dangerous allocation. Count
  the representation that actually grows, not only source steps.
- Charge immutable retained buffers, active scratch, foreign allowances and
  concurrency reservations at their actual ownership lifetimes. Avoid charging
  each prepared object a full active workspace for its lifetime. Keep a documented
  conservative foreign allowance where the library cannot report usage.
- Expose typed configurable math/resource policy through engine/Python settings.
  Start from the existing relaxed workstation profile; use effective process
  quotas and nested-library concurrency in admission. Resource refusal is typed
  and does not silently truncate work.
- Assemble fit Jacobian/Hessian structure through library sparse representations
  where the true support warrants it. Dense fallback remains an explicit choice
  for genuinely dense problems; sparsity cannot be assumed from problem size.

**Targeted checks:** representative 20/44-coordinate derivative admission,
bounded repeated-let growth, multiple prepared cases versus active work,
oversized-resource refusal before allocation, lease release after cancellation,
sparse/dense small-case numerical equivalence and larger sparse fitting admission.

**Delete:** disproven size estimates, duplicate/unowned reservations and dense-only
assembly limitations replaced by the selected sparse path. Report reservations
and measured RSS separately in P18; no new restrictive global memory ceiling.

### P15 — Remove repeated case work where the contract permits it

**Responsibility:** native preparation, case assembly, dynamics callbacks and
constant/coefficient evaluation; F28 and associated ledger candidates.

- Avoid repeating structural analysis only when an immutable admitted product
  establishes the exact assumptions required by the consuming adapter. Recheck
  changed structure or value-dependent properties; a layout stamp alone is not
  a blanket admission certificate.
- Complete checked rebind APIs and immutable-body sharing from P11. Precompute
  semantic-ID-to-slot maps for hot callbacks rather than repeatedly joining by
  map lookup; retain diagnostic mappings outside the hot representation.
- Use library constant evaluation instead of constructing an optimized evaluator
  for each constant, preserving numeric/domain semantics. Share coefficient
  activity calculation where it is the same operation, while keeping original-
  model verification independent of the solver upload.
- Prefer a qualified direct JVP operation where available; otherwise reuse
  Jacobian work only under a key covering every varying time/state/parameter/
  mode/provider dependency and validity condition. Retain scratch at the attempt
  boundary instead of allocating a matrix on every callback.
- Use targeted counts/profiles to choose candidate changes; full comparative
  timing remains P18. Drop a speculative optimization if it adds complexity
  without demonstrated value. Record that disposition rather than claiming an
  assumed speedup or adding an unneeded cache.

**Targeted checks:** clean/reused numerical agreement; structural-analysis counts
over a value-only sweep; correct invalidation on structural/parameter/mode edits;
JVP agreement and invalidation; no new result dependence on allocation reuse.

**Delete:** the corresponding per-call reconstruction, deep copies and superseded
constant evaluators after the consumer uses the admitted/shared product.

### P16 — Reconcile the remaining deletion and retention candidates

**Responsibility:** the concrete F21/F23/F24 candidates across engine, catalog,
authoring, kernels, structure and their documentation; no blanket purge.

For each row in the disposition table below, establish the surviving behavior and
owner from callers and the target journeys. Remove obsolete bespoke mechanisms and
their tests/fixtures in the same change. Retain an operation when it supplies a
required authoring, inspection, storage or library capability; repair false claims
about its role. Required-but-unwired model contracts are consumed or explicitly
refused through P03/P04, not deleted just because they were previously ignored.

| Candidate set from F21 | Required disposition |
|---|---|
| Engine round epochs/reusable preparation, provider-policy requirement subsystem, obligation receipt cache, `ScopedPool` | Remove obsolete rule/execution orchestration; retain only behavior with a distinct surviving contract. Do not rebuild a general rule engine. Shared admission/resource behavior moves to its existing owner |
| SQL workbench DDL/DML/capture/resolution, `operation/ports.rs`, `plan_codec.rs` | Retain supported authoring/inspection/storage operations through native DataFusion/catalog owners; remove duplicated frontends and serialization lacking an actual contract |
| Constant null/kernel-outcome lineage and dependency receipts captured on every publish | Remove claims and recurring work that no consumer needs; preserve exact dependency/version information required to reopen, settle or interpret publication |
| Catalog maintenance, retention, CDF, DML, checkpoint/consumption | Preserve the selected publication/reader/retention protocol and useful library operations. Remove obsolete orchestration, not the guarantees P12 needs |
| `pse.canon.v2` and unconsumed generated relation families | Replace overlapping identity/canonicalization through P02/P12. Classify each family as target meaning, storage/tooling contract or obsolete mechanism before removal |
| Structural `domains.rs`; physical opcode vocabulary, smoothing, `ScalarKernel`, `InferenceCache` | Retain semantic domains and admitted formulation policy; remove unused bespoke execution/cache paths replaced by current library owners. A vocabulary must not advertise unsupported execution |
| YAML-as-UDF parsing, rename/`apply_edits`/`assign_ids`, B-tree-internals heap estimator | Keep a coherent model-authoring/edit path; remove duplicate parsing/estimation and implementation-detail assumptions. Do not remove useful editing merely because its caller is tooling |
| `pse-rules` and rule/package projections | Establish actual consumers, including xtask fixture validation. Remove or retain that role honestly; any crate removal follows P00's ADR route. Correct ADR/living-document claims of product inspection |

**Targeted checks:** surviving model edit, inspection, publication/retention and
fixture-validation behavior; compile affected callers. Symbol searches confirm a
named retired mechanism has no callers, but do not substitute for behavioral
checks or prove architectural alignment.

**Completion:** every row has a concrete implemented removal/replacement or a
reasoned retained contract. A TODO to reassess it later does not close the packet.

### P17 — Simplify qualification tools and make evidence honest

**Responsibility:** just recipes, validation/phase/review scripts, nextest/pytest
configuration, governance checks and evidence documentation. This packet can run
after P00, before the product packets, once its governance decision is approved.

- Remove plan-specific guards from ordinary `test`, `codegen`, `bench-smoke` and
  development commands. Keep any qualification precondition on the actual case
  measurement campaign, with errors that explain missing functional evidence.
  Avoid encoding Plan 16 as the next hard-coded plan in the same framework.
- Use nextest profiles/filtersets and pytest collection/JUnit for selection and
  execution facts. Preserve exact required native/Python coverage and empty-
  selection failures; do not maintain test names independently in manifest,
  recipe and CI. Retain small orchestration only where it serves a real operation.
- Record executed inputs/tool/native identities and outputs. Replace inferred
  receipt continuation with explicit executed, unchanged-input reuse, reviewed
  transfer and not-run labels. A source snapshot includes relevant configuration,
  generated/native inputs and nonignored dirty/untracked files, with explicit
  treatment of external/ignored dependencies. A Git tree alone is not an
  applicability proof. Never relabel historical tests as fresh execution.
- Remove digest-bound reviewer-string verdict gates and mandatory reissue loops.
  Cite review documents as judgment; keep useful raw test/benchmark reports.
  Do not build automated design-conformance or impact-inference tooling.
- Make opt-in `policy` strict for its actual group while keeping advisory reports
  advisory. Separate clock-driven deferred-trigger checks from deterministic
  ADR/schema lint. Use Cargo manifests as pin authority and one enforcement
  owner per ban kind (type-aware Clippy or structural rules as appropriate).
  Replace string-absence “architecture witnesses” with behavior where needed.
- Qualify a supported Criterion output interface before replacing private JSON
  parsing; the original CSV claim is unverified. Replace custom setup reporting
  with ordinary test reporting where it preserves isolated stdlib setup tests.
  Remove the nonworking release stub/recipe unless an actual release workflow
  justifies a library-owned replacement; release automation is not a new product
  deliverable here. Retain historical reports and use explicit opt-in retention
  for future report cleanup; never delete Rust caches to simplify evidence.
- Provide plan-neutral public recipe names for retained journeys: **proposed**
  `just native-test`, `just native-python <output>` and `just case-measure <output>`
  replace the corresponding Plan 14-specific entrypoints. Preserve their pinned
  features/native environment and update consumers in the same change. These
  recipes are ordinary execution commands, not a new acceptance framework.

**Targeted checks:** source/config/native change cannot masquerade as unchanged
inputs; documentation-only change does not force solver reruns or review reissue;
strict/advisory exit codes; a fixed source's lint does not fail solely because
tomorrow arrives; empty test selection fails; ordinary recipes run without phase
receipts; benchmark samples are read through the qualified output contract.

**Delete:** replaced phase/receipt/verdict plumbing, duplicate selectors/ban lexers,
stale recipe callers and unsupported release surface. No arbitrary runner-size or
receipt-count target. Do not erase M22's records.

### P18 — Qualify once, review the final architecture and close

**Responsibility:** complete selected-scope acceptance, case measurements,
repository checks, final independent review and evidence-labelled documentation.

1. Confirm implementation and deletion completion for P01–P17, then run the full
   qualification table below. Environment repair is warranted only for a reported
   blocker; refresh the editable native extension when changed Rust requires it.
2. Repair observed failures at their owner and rerun affected checks. Required
   local baseline is zero; record upstream/excluded scope separately rather than
   describing it as product success. A newly discovered required path cannot be
   dropped merely to obtain green counts.
3. Measure the complete case campaign with already-built applications and preserved
   Rust caches. Compare only like-for-like physical/numerical conditions; use
   existing M22 measurements as historical context when inputs differ.
4. Request an independent final review, without keeping an agent running during
   implementation. Review G1–G8 and PS-G1–PS-G3 separately against actual code and
   observations, plus the F/N coverage tables. An unresolved required gate is not
   acceptance. Review findings cause focused repair, not a new ceremonial receipt.
5. Reconcile current blueprint/ADRs, public capability docs/examples, active plan
   indexes and actual surviving consumers. Record Outcome with what was built,
   a mistake corrected and deliberate deviations. Do not edit historical test
   outcomes to match the new design. Commit/push/merge is not implied by this plan.

**Completion:** all mandatory target journeys, removals/dispositions and gates are
settled with scoped evidence; no task-owned process remains. Optional library
experiments may be rejected with a reason; required model behavior may not remain
unimplemented behind a generic “future work” statement.

## Coverage of the two reviews

This is a planning traceability table, not another machine-enforced evidence
manifest. Update dispositions as implementation settles them; exact commands and
results belong to ordinary test output and the final Verification/Outcome.

| Original finding | Aggregate action | Packets | Acceptance focus |
|---|---|---|---|
| F01 | Implement real flow/tear/strategy and native QP/conic/fixed-point routes; repair latent scope, fan-out, forbidden-tear and selected-revision defects; use contextual strategy | P03/P06–P08 | Recycle by public API, recorded history, each claimed class routable |
| F02 | Complete selected-scope admission and species/material/reaction binding; refusal is explicit for unsupported physics | P03/P04 | No ignored required declaration; permutation/element closure |
| F03 | Preserve exact power-literal semantics | P01 | Analytic authored power cases |
| F04 | Safe explicit library initialization, environmental capability and declared determinism | P01/P02/P11 | Subprocess containment and stated cross-process equivalence |
| F05 | Per-class policy and independently qualified native outcomes, including acceptable/gap/resource states | P05/P07/P10 | Native status, KKT, MIP gap and original-space checks |
| F06 | Stable typed durable outcomes; useful candidates/derivatives with honest qualification | P02/P10 | Limit/rank/round-trip and capability-settings cases |
| F07 | Consume authored scaling and nominals; record algorithmic defaults | P05 | Unit scaling, policy precedence and difficult-scale solve |
| F08 | Complete physical kind/reference/data/phase binding; validate entropy claim and declared stability policy | P04 | Independent entropy and incorrect-kind/state cases |
| F09 | Normalize tolerances/transforms, preserve bounds/guards and explicit closure policy | P05 | Mixed-unit infeasibility and original-space acceptance |
| F10 | Shared time-coordinate and heat/work meanings | P02/P04/P09 | Offset-time fit and separate heat/work results |
| F11 | Semantic projections, common float equality/framing and retained-state reconfiguration | P02/P11 | Output tolerances, sparse starts, KINSOL settings, signed zero |
| F12 | Semantic compatibility distinct from prose/build/encoding | P02/P12 | Complete contract closure and compatible reopen |
| F13 | Actual start provenance independent of allocation reuse | P08/P10/P11 | Sequence root/basis provenance and no-start reset |
| F14 | Immutable-member parent/attempt/settlement protocol and read-only reopen | P12 | Republish, conflict, interrupted settlement, retained readers |
| F15 | Scoped overlays and separate original/solved values | P08 | Failure/cancellation cannot change the specification |
| F16 | Structured error classes, source/structural/presolve/provider witnesses through Python | P02/P06/P10/P13 | Classification and attributable diagnostics |
| F17 | Qualified Symbolica interval projection and one affine/obligation fact | P06 | Transcendental FBBT and guarded affine consistency |
| F18 | Admitted requirements, one scoped capability owner, complete fit threading | P02/P06/P07/P09 | Early nonsmooth/bound/thread routing decisions |
| F19 | Actual growth bounds, correct lifetimes, configurable generous policy and sparse fitting | P14 | Large derivative/sparse-fit admission and bounded growth |
| F20 | Explicit trial/formulation/event policy and qualified hybrid sensitivities | P09 | Back-pressure, recovery, scheduled/state-triggered evented fit |
| F21 | Resolve every deletion candidate by surviving target contract; remove fictitious claims | P10/P16 | Supported authoring/inspection/storage journeys and removed callers |
| F22 | Replace redundant generated copies only with complete consumer-bound contracts; preserve useful typing | P13 | Contract mismatch, enum and fixture-discovery behavior |
| F23 | Consolidate equal cache mechanics; preserve retention fences, native lifetimes and semantic Salsa bounds | P11 | In-flight clear/cancel/retry, storage invalidation and rotation |
| F24 | Consume owner products for order, obligations, grammar, vocabulary and traversal | P06/P10/P13 | Stable identities and equivalent owner/consumer obligations |
| F25 | Honest evidence applicability, complete inputs, strict policy and clock-free lint | P17 | No stale-result relabeling; correct command semantics |
| F26 | Remove alignment ceremony/phase guards and duplicated authority; ordinary runners and behavioral checks | P00/P17 | Useful recipes independent of plan receipts; manifest pin authority |
| F27 | Correct active architecture/consumer descriptions through accepted-authority routes; preserve M22 history | P00/P16/P18 | Coherent current blueprint, ADRs and public claims |
| F28 | Eliminate repeated work only under complete admitted dependencies and measured benefit | P11/P15/P18 | Case costs and work counts, clean/reused equivalence |

| Follow-up finding/addition | Plan obligation | Packets |
|---|---|---|
| N01 | Complete enforced interpretation at the existing admission boundary | P03/P04 |
| N02 | Complete species/parameter/port/reference/formulation binding | P04/P09 |
| N03 | One resolved numerical/eligibility contract with distinct capability scopes | P02/P05–P07 |
| N04 | Strategy, overlay, start and time-coordinate coherence | P03/P08/P09 |
| N05 | Deliberate semantic identity projections and complete reuse dependencies | P02/P11/P12 |
| N06 | Qualification as domain data; no blanket loss of valid candidate derivatives | P07/P09/P10 |
| N07 | Actionable model diagnostics through all representations | P02/P06/P10/P13 |
| N08 | One publication/settlement/compatibility/retention protocol | P12 |
| N09 | Contract-based simplification with resource and cache lifetime distinctions | P11/P13/P14/P16 |
| N10 | Evidence applicability distinct from source identity; ordinary tooling | P17/P18 |
| R2 ordinary-extension target | Compact templates, valid domains, typed connections, shared physics and inspectable bindings | P03/P04/P09/P18 |
| R2 authority/effect/library uncertainties | Resolve G1/G4/G8 with actual owner/effect/library evidence; no assumed pass | P00/P01/P11–P18 |

### Unnumbered library and smaller recommendations

Each row receives an implementation or a documented contract-based non-adoption.
An unverified API name in a review is a candidate, not an implementation command.

| Recommendation set from R1 §8 / R2 §8 | Disposition and owner |
|---|---|
| Git source snapshots, nextest selection/replay, pytest JUnit/collection, small report runner | P17: adopt ordinary tools where qualified; preserve applicability distinctions, no fresh-suite-per-doc-edit rule or 200-line target |
| Criterion CSV/cargo-criterion output | P17: verify exact supported interface and select it; do not assume the asserted CSV feature exists |
| cargo-release/release-plz/git-cliff | P17: remove nonworking release stub unless a real retained release operation warrants a qualified library replacement |
| strum/re-exports, runtime fixture harness, complete semantic digests | P02/P12/P13: contract-preserving consolidation with compile-time and test-discovery guarantees |
| Fenced caches, cancellation token, neutral flight ownership, Salsa durability/interned-key accounting | P11: qualify exact lifecycle semantics, preserve late-insertion/resource fences, avoid unnecessary crate creation |
| Sparse fit assembly, sparse coefficient activity and direct constant evaluation | P14/P15: library-owned algebra, shared equivalent operations and independent original-model validation |
| Native status constants and actual library-version APIs | P01/P07/P10: stable semantic mapping, actual environment provenance, no literal version masquerading as loaded identity |
| Numeric PSD factorization/eigenvalue assessment | P06/P07: typed approximate certificate alongside the exact path, no silent convexification |
| FeOS parameter readers and component identifiers | P04: checked species/data mapping and reference convention; API choice follows pinned-contract validation |
| Source path/span libraries, DSL target parsing, exception presentations | P02/P10/P13: one structured diagnostic/grammar owner with useful locations |
| Delta log/action/transaction readers | P12: adopt if size bounds, exact-version and settlement evidence remain intact |
| Ipopt acceptable/bound/scaling/warm-start options; HiGHS gaps/basis; KINSOL strict signs/scales/history | P05/P07/P08: derived native projections of explicit domain policy |
| Cycle witnesses and fine structural decomposition | P06/P08: consume existing proofs first; add library graph refinement only for needed model diagnostics |
| Diffsol reset sensitivities/adjoints, optional IDA/IDAS trial recovery | P09: qualify scheduled/state-triggered reset semantics; adjoints or another integrator only for an operation whose shape needs them |
| FeOS derivative/stability/flash/contribution APIs | P04/P09: selected phase/reference contract and honest physical diagnostics, not an unconditional hidden filter |
| Symbolica symbols, initialization capabilities and block-local Numerica jets | P01/P06/P14: supported lowering, safe runtime entry and sound growth admission |
| DataFusion tracing/completed-plan metrics and typed spill compression | P10/P13: observable retained execution and typed configuration without duplicate work |
| Kept custom units, fallible Arrow ownership, durable row tokens, shared-DAG traversal, exact tear MILP, Clarabel cone helpers, direct HiGHS/SuiteSparse bindings and guarded provider execution | Retain the reviewed semantic reason at the owner; P00/P13/P16 recheck only if that reason changes. Do not replace merely for line count or library branding |

## Verification

**Proposed.** No implementation acceptance or performance result is claimed by
this plan. “Interface-checked” findings in the reviews remain at that strength
until the named behavior executes. A final claim of **Tested** or **Measured** names
its command, mode, conditions and zero-failure baseline; counts alone do not prove
coverage. Unavailable observations are not recorded as zero.

### Public acceptance journeys

Targeted tests are added during their owning packets. The complete journeys below
execute together in P18, including relevant negative and interruption cases.

| ID | Journey / independent oracle | Required evidence |
|---|---|---|
| V01 | Authored powers and guarded math against analytic values/derivatives | Supported syntax survives public admission; no guard lost by simplification |
| V02 | Symbolic environment and determinism in isolated processes | Unsupported licensing returns a typed outcome; declared reproducibility class holds without hidden tracked effects |
| V03 | Builder/document/template extension with irrelevant stored data and unsupported requested data | Same meaning admitted; silent loss impossible; ordinary unit variant needs declarations/bindings only |
| V04 | Species permutation, quantity-kind/reference checks and independent entropy/property cases | Correct data correspondence and declared reference agreement; invalid binding rejected |
| V05 | Executable balanced reaction and deliberate stoichiometric/energy inconsistency | Element/material/energy closure or attributable admission failure |
| V06 | Equivalent units and mixed-magnitude constraints, bounds and scales | Meaning-preserving presolve/postsolve and explicit physical acceptance |
| V07 | Actual native class routes and unacceptable/acceptable/gap/limit/resource exits | Contextual capability matches public behavior; qualification follows checked evidence |
| V08 | Tear-driven recycle, constrained block and failed continuation | Correct strategy/history; forbidden tears refused; original specification unchanged |
| V09 | Multi-root sequence and standalone case with explicit seed/no-seed policies | Used start explains changed root/basis; native reuse does not imply hidden warm start |
| V10 | Nonzero-origin dynamics/fitting, back-pressure, scheduled and state-triggered reset examples | Correct coordinates, conservation and derivative validity; unsupported event cases explicitly refused |
| V11 | Failed/rank-deficient fit, partial trajectory and useful candidate derivative | Typed outcomes and diagnostics survive Rust/Arrow/Python round trips |
| V12 | Infeasible/ill-posed/provider/parser failures with independently known offending entities | Stable classification and source/unit/port/witness attribution |
| V13 | Clean versus reused preparation after float, profile, parameter, mode and provider edits | Complete dependency invalidation; correct retention across unrelated storage work |
| V14 | In-flight clear, retiring flight, cancellation, destruction and resource limits | No stale insertion contrary to clear policy; ownership through join; no memoized cancellation or silent truncation |
| V15 | Parent-chain publication, race, partial write, uncertain commit and settle | Exact control-last visibility, stable outcomes, no poisoned base |
| V16 | Read-only and old-contract reopen, doc/encoding versus semantic change, retained readers | Explicit compatibility and retention; no read-side writes or accidental data loss |
| V17 | Generated/compiled contract mismatch, nested/composite obligations, grammar and fixture discovery | One semantic owner with equivalent consumer behavior and complete discovered tests |
| V18 | Tooling source/config/native changes, strict/advisory groups and ordinary commands | Honest applicability, complete execution selection and no alignment ceremony |

### Final command surface

P17 must leave these as ordinary, documented recipes. The three new names below
are proposed replacements, not commands asserted to exist at plan creation.
Reuse one aggregate invocation only if it covers this exact scope without running
the same suite twice.

| Scope | Command / mode | Completion condition |
|---|---|---|
| Rust workspace | `just test --no-fail-fast`; `just doctest` | Full default workspace and doctests, explicit force-validation; zero failures |
| Linked native routes | Proposed `just native-test` | Required native feature graph and all V01–V18 native/component journeys; zero failures, no missing-case skips |
| Python public boundary | `just py-sync-native`, then proposed `just native-python <output>` and the remaining `just py-test` scope | Fresh linked extension; unit/component/integration paths and public workflows, no duplicate selection |
| Physical references | Existing analytic/exhaustive/property reference harness extended for V04/V05/V10 | Independent reference inputs and declared tolerances; no IDAES parity claim from shared tests |
| Case performance | Proposed `just case-measure <output>` | Complete scenario set below after functional qualification; no Rust build time in measurements |
| Rust/repository quality | `just fmt-rust-check`, `just clippy`, `just lint-typos`, `just lint-license`, `just quality` | Required checks green against zero baseline; aggregate deduplication permitted |
| Generated/governance contracts | `just governance`, `just codegen-check`, `just architecture-manifest` after P17 removes stale plan coupling | Current generated/source mappings and family/governance invariants; no new design-verdict framework |
| Decisions and docs | `just adr-lint`, `just adr-index`, `just lint-agents`, `just docs` | Current decision/index/links and book; run index generation before its final equality check |
| Reference parity, if claimed by the implementation | `just parity-container` | Actual exercised parity against the repository pin; missing environment is not a passing skip |

The prior M22 waiver of broad strict-Clippy cleanup is a historical scope decision,
not a claim of a clean baseline here. Resolve affected and required final findings
once at plan end. Do not divert into the known upstream `proc-macro-error` warning
as a new remediation project; report that upstream condition separately without
mislabeling it as a local clean-toolchain result. Exhaustive features, release
builds and distribution certification remain outside this local target.

### Case rebuilding and performance conditions

- Preserve checkout, target directories, compiler caches and native prerequisites.
  A necessary rebuild is permitted as **untimed setup**. Do not run `cargo clean`,
  disable compiler caching, change worktrees/toolchains or time complete Rust
  compilation as a case benchmark.
- Cold means a fresh process-case/runtime/compiler owner in an already-built
  application. Warm means retaining applicable model/compiler/artifact state.
  Record start policy separately from native allocation reuse.
- Retain the useful 23-case M22 workload shapes and add the new target cases:
  declared recycle strategy, mixed-scale solve, explicit QP/conic path, 1,000-point
  value sweep, dynamic parameter/horizon rebind, supported evented fit, sparse
  fitting growth, repeat publication/settlement and cancellation/resource teardown.
  Consolidate overlapping scenarios rather than duplicating campaigns.
- Compare like-for-like model inputs, policy, profiles, library versions and thread
  counts. Use existing binaries/reports as before-evidence only when their inputs
  support the comparison. A baseline that lacks a new capability is “unavailable,”
  not a fabricated performance ratio; M22's historical total is not a fresh run.
- Record phase/end-to-end time, samples/uncertainty, structural/evaluator build
  counts, allocations or reservations where available, and independent process
  peak RSS. Include attempt teardown and report retained versus active memory.
  Keep generous workstation limits and document effective quotas/other active
  work. No invented absolute timing threshold or universal speedup requirement.
- An optimization from P15 lands only with preserved behavior and measured value
  or an explicit simpler-ownership benefit. If measurements expose a regression,
  repair or remove that optimization, then rerun its affected cases.

## Decisions to settle within packets

These are bounded implementation decisions with owners, not reasons to stop
planning or ask for approval of unfinished work.

| Decision | Settling condition | Deadline |
|---|---|---|
| Restricted Symbolica execution and determinism guarantee | Pinned capability/failure contract plus isolated execution; typed refusal is the safe default for unsupported concurrency | P01, before symbolic jobs use it |
| Exact enum/identity/diagnostic owners and retained-result compatibility | Dependency direction, semantic projection and representative round trips | P02, refined by P12 before storage changes |
| Entropy convention and selected phase/formulation behavior | Explicit physical reference, independent comparison and property API contract | P04 before new property claims |
| Derivative/class and approximate convexity evidence | Actual normalized library representation and original-model policy | P06 before P07 selection |
| Integrator recovery and hybrid sensitivity route | Required example plus pinned event/reset contract; another library only if it serves that requirement | P09 before evented fitting is advertised |
| Cache consolidation, flight placement and resource estimates | Matching lifecycles, package DAG and allocation/growth evidence | P11/P14 before removing guards or accepting larger work |
| Frozen-contract and fixture-harness replacements | Complete consumer binding and stable discovered cases | P13 before deleting generated checks |
| Every F21 removal and optional ledger adoption | Concrete surviving behavior/library fit, or documented non-adoption with no false capability claim | P16/P17 before final qualification |

## Risks and controls

| Risk | Control |
|---|---|
| “Data-based” becomes another generic framework | Extend existing contracts; use the ordinary-unit extension journey to test locality; keep library algorithms behind thin boundaries |
| Fixes change the scientific problem implicitly | Make units, references, phase, smoothing, convexity and tolerances explicit; verify original-space physical outcomes |
| A simplification removes required behavior or ownership | Delete only after the replacement is targeted-tested; preserve reader, attempt and cache-retention lifetimes |
| New typed outcomes strand consumers or stored results | Move callers and generated views together; explicit stored-contract compatibility/refusal, no parallel execution engine |
| Large scope becomes repeated acceptance bureaucracy | Targeted packet tests; one P18 campaign; affected reruns only; final independent review, no standing review agent |
| Performance work destroys compiler reuse or imposes tiny memory ceilings | Stable paths/caches, untimed builds, configurable generous resource policy and separate case/RSS measurements |
| Historical evidence is overstated or erased | Preserve M22 records, name applicability, and state new gaps/results prospectively |

## Outcome (recorded after implementation)

### What was built

**Implemented:** P00–P04 foundations and their supported boundary are recorded in
[the execution packet](16-p00-p04-execution.md). P05–P06 numerical policy and shared
mathematical/structural facts are recorded in the
[numerics/facts packet](16-p05-p06-execution.md). P07–P09 contextual native qualification,
explicit starts/strategies and shared dynamics/fitting are recorded in the
[native strategies/dynamics packet](16-p07-p09-execution.md).
**Proposed:** P10–P18 remain pending.
Full-plan qualification and performance evidence belong to P18; the scoped packet's
regressions do not establish whole-plan acceptance.

### A mistake made and corrected

The P00–P04 packet records corrections to multi-case balance identities, material
policy dependencies, vessel coordinate ordering and generated invariant fixtures.
The P05–P06 packet records the builtin-constant projection fix, final caller migration
and corrected native/generation qualification environment.
The P07–P09 packet records native-status overclaims replaced by original KKT/gap
qualification, complete fixture generation and final feature/boundary repairs.

### Deviations from the plan, deliberate

P01 uses the existing personal license configuration without a new licensing or
process-isolation framework. P04 admits homogeneous explicit-density providers and
explicit-heat molar reactions; unsupported formulations are refused. See the scoped
packets and proposed ADR-0088–0093 for the detailed boundaries and decision PR route.
P09 admits a narrow smooth IDAS recovery profile alongside Diffsol's qualified hybrid
route. Presolve multiplier recovery and QP regularization can leave feasible candidates
without stronger optimality qualification; the explicit alternatives are tested.

[R1]: ../design_review/reviews/design_review_comprehensive-codebase_2026-09-24.md
[R2]: ../design_review/reviews/design_review_data-model-architecture-followup_2026-09-24.md
[CH]: ../design_review/design_principles/DATA_MODEL_DESIGN_CHARTER.md
[ST]: ../design_review/design_principles/standard.toml
[M22]: 14-m22-execution.md
