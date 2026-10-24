# pse-arrow binding

The repository layer of the design standard. It defines no principles; it maps the
[core principles](../core/design-principles.md) and the
[process-simulator profile](../profiles/process-simulator/principles.md) onto this
repository's authorities, commands, routes and local policies. Where this page and a cited
authority disagree, the cited authority wins and this page is corrected. Those authorities govern
plan execution and implementation review in their domains; a design review treats them as current
design and reports its rule impacts instead (design-review, Grounding).

## Standard applied

[`standard.toml`](../standard.toml) owns the selected versions and paths. Read the core
principles/template and applicable profile from that declaration; this binding does not copy
the version values. The core's version-transition table preserves historical DP/G references.
AP-04/G9 require an adequate, explicit domain model that governs behavior, independently of
the remaining architectural and scientific judgments (ADR-0129; blueprint §24.4).
This assessment belongs to the bounded reviews below. Ordinary implementation work does not
itself trigger a domain-model review; investigation depth and optional flow tracing follow the
concrete question being assessed.

AP-07/G9 qualitatively assesses execution fit alongside those foundations, under
[ADR-0162](../../../adr/0162-workload-fit-review-principles.md). Use the simulator's declared
edit/re-solve, study, recycle and dynamic workloads; scope size/skew, concurrency and resource
premises where material. Semantic unit/model boundaries do not dictate physical batches,
allocations, transactions or native calls. Compare complete preparation/evaluation/solve and
publication/recovery work, including immutable reuse, compact views and nested pool contention.
Explain material tradeoffs in plain language. This consideration requires no numerical estimates,
cost model, estimator, runtime cost accounting, execution-planning machinery, instrumentation,
formal cost proof or additional proof artifact; those mechanisms need a separate concrete
functional or operational requirement. Quantitative performance/capacity claims still require
measurements. Qualitative reasoning can defeat acceptance without a benchmark; no new review cadence, checklist,
probe mandate, lint or product qualification is introduced by this adoption. Scientific gates
and PS-09's solver/composition ownership remain independently required.

Use the [core principles](../core/design-principles.md) together with
[Heuristics for Efficient Architecture](../core/efficient-architecture-heuristics.md), selected
by `[core].heuristics`, for consequential choices. The companion maps to AP-07 and existing G9,
not a new architectural judgment. Consider relevant execution patterns before physical
organization, interfaces, preparation, assurance and lifecycles become fixed; address material
mismatches early. During implementation, focus on choices left open or exposed mismatches
without restarting settled reviews. Existing principles, twelve lenses and overlapping guidance
remain; the companion introduces no exhaustive checklist, cost models or proof machinery.

## Design principle audit

The [audit skill](../../../../.codex/skills/design-principle-audit/SKILL.md) maintains
[one living Markdown collection](../../audit/README.md) at `docs/design_review/audit/`.
It diagnoses current code broadly against the functional target and the same core, heuristic and
profile principles, without an incoming library preference or principle subset. It need not choose
remedies or render an overall acceptance verdict.

Refresh the account before a new diagnostic design-review engagement about production or broad
system scope, including direct design-review invocation. Test-only engagements use the separate
test account below; mixed engagements identify which accounts cover the consequential questions
and refresh the applicable scope without claiming that one baseline refreshes the other.
Use the preceding audited source state and intervening changes to focus current inspection;
reconsider unchanged dependencies and earlier findings when their premises change. Record source
commit, included dirty changes, date, standard versions and meaningful coverage limits honestly.
The publishing commit is distinct from the source baseline. An empty scaffold is not an audit.

One refreshed account can serve preparation and review in the same engagement while its baseline
remains applicable; reconcile intervening source or standard changes before relying on it. Reviews
cite its publication revision and stable finding anchors. An inherited observation keeps the date
and applicability limits of its evidence rather than becoming freshly verified by republication.

The audit owns baseline diagnosis and groups structural causes in flexible documents, initially
around semantic authority, structure/invariants, boundaries/composition, local reasoning/assurance
and execution fit. Give each finding one primary home, relevant principle IDs and its inspected
footprint; distinguish suspected reach. Keep identity across moves and material lineage across
splits/merges. Resolve and remove findings on evidence; Git and the removal commit preserve history.
Do not copy plan progress or run a new whole-codebase audit to close every implementation slice.

Ordinary implementation, focused plan-foundation assessment and loading review criteria do not
initiate a broad principle audit or formal review. Principle audits do not automatically start solution reviews or authorize
remediation. Process discussion and edits to this workflow do not require an audit/review artifact.
The initial audit and detailed library overviews remain separately scoped work.

## Test code design principle audit

The [test audit skill](../../../../.codex/skills/design-principle-audit-tests/SKILL.md) maintains
[a separate living collection](../../test-audit/README.md) at docs/design_review/test-audit/.
Use it for an explicitly requested test-scope audit and before a new diagnostic solution review
whose subject is test architecture or test evidence. This is a sibling diagnostic route, not a
whole-production audit prerequisite. Ordinary test edits, implementation reviews, qualification
runs and installation of this process do not initiate it.

Load the [test-assessment companion](../core/test-assessment.md) through `[core].test_assessment`
alongside the selected core, heuristics and applicable domain profiles. It supplies test-specific
judgment, not new principles, gates or a mandatory test campaign. Tests include relevant inline
code, fixtures, helpers, generators, doubles, oracles, reference data and execution configuration;
production paths are inspected as needed to understand contracts and assurance dependencies.

The test account records its own audited source state, included dirty changes, date, standard
and companion versions, inspected scope and limits. Its publishing revision is distinct. Use
intervening production-contract and support changes as well as test edits when refreshing it;
an unchanged test can have changed evidential meaning. Neither an empty scaffold nor another
account's source baseline establishes test-audit coverage. Reviews cite the applicable publication
revisions and finding anchors and reconcile intervening changes before relying on them.

New test findings have stable DTA-nnn identities, beginning with DTA-001. Each structural cause
has one primary finding home across both audit collections. Existing DA-nnn findings retain
their identity and home and are linked rather than copied; reexamination here states its scope
without silently refreshing that other account. Preserve lineage on moves, splits and merges,
never reuse retired IDs, and remove resolved material only on evidence with recoverable history.
Reviews own alternatives and judgments; plans own scheduled disposition and closure evidence.
This collection diagnoses and does not select remedies, authorize repairs or certify suite quality.

## Functional capability exploration

Use `functional-extension-review` (source: `.codex/skills/functional-extension-review/SKILL.md`)
and its preparation companion for deliberate functional enhancement within the operator's chosen
outcome, workflow, subsystem or ecosystem. The [functional collection](../../../functional_reviews/README.md)
owns output and recommendation-audit conventions. These reviews assess useful capabilities and
their realization without a diagnostic audit prerequisite or required principle violation.
The diagnostic design-review cadence below remains distinct. Selected opportunities enter the
existing planning and decision routes; a review does not itself approve scope or implementation.

## Plan improvement

Use `plan-improvement-review` (source: `.codex/skills/plan-improvement-review/SKILL.md`) at
requested or explicitly planned boundaries to assess the remaining change footprint of a plan
or coordinated series. Include design-principle alignment and general enhancements to components
substantially modified to accommodate the plan, even when their value extends beyond its
functional target. Apply the selected core and process-simulator profile, preserving independent
architectural and scientific/model-adequacy judgments. Judge recommendations for coherent
integration and total burden. Publish a standalone
[plan-improvement review](../../../plan_improvement_reviews/README.md) with proposals distinct
from accepted scope and scheduled work. Plan revision from that document is separately
instructed; no separate update skill is required.
This workflow has no broad diagnostic-audit prerequisite, does not refresh an audit account and
does not replace a formal design review due at the cadence below. Existing scope and decision
routes govern adoption.

## Reviews in this repository

- **Location:** `docs/design_review/reviews/design_review_{slug}_{YYYY-MM-DD}.md`; supporting
  evidence files under `docs/design_review/evidence/`.
- **Skills:** `design-review` (core method) with `design-review-process-simulator` (profile
  lenses). The `design-reviewer` agent applies both with this binding.
- **Focus:** the operator selects holistic/unconstrained, library-targeted, principle-focused
  or combined review. Clarify an unspecified selection; agents choose methods and depth within it.
  Library focus is an investigation hypothesis, not required adoption. Relevant guarantees still
  constrain remedies; encountered out-of-focus concerns do not automatically expand solution scope.
- **Inputs:** cite the refreshed diagnostic-audit revision and stable finding anchors. Inspect
  decisive current sources, compare corrections and preserve useful strengths. An applicable audit
  can serve preparation, review and recommendation audit within the same engagement.

- **Grounding:** the standard and the functional target. The target is stated as outcomes in
  the process-simulator profile's *Functional target* (a best-in-class simulator serving
  edit/re-solve, studies, recycles, dynamics and extension) and in blueprint §0 and §0.2 (the
  IDAES-PSE modeling capability families this core re-implements). The mechanisms those passages
  name are rules, like ADRs, blueprint D1–D14, plans and the policies on this page: the Rust core
  with a Python authoring and result boundary, the clean-room parity harness against a pinned
  `idaes-pse`, and §0.2's *Realization here* column. A review lists the rules its
  recommendations would change (`review#RCnn`); they change only after the operator confirms
  them in plan creation.
- **Purpose:** every review, design or change tier, has the **target** purpose (template,
  *Tier and purpose*): the aim is the best-in-class simulator design. The template's
  conformance purpose is not used; whether an implementation matches its accepted design
  belongs to implementation review.
- **Reviews are evidence, not authority.** Their findings take effect through a plan, an ADR or
  a `design:` PR, once the operator has confirmed any rule change they carry.
- **Lifecycle:** a review stays in the tree while an open finding or a pending decision depends
  on it. Once findings are dispositioned and enduring rationale has its owner, it retires to Git
  history; retained ADRs cite it as `git:<commit>:<path>` (ADR-0096).

ADR-0156 supplies the reviewed ProcessSimulator 1.4 PS-09 clarification: native libraries
own fitting numerical iteration, globalization, factors and model management; domain owners
supply explicit scientific families and bounded composition only where no fitting library
supplies the complete consumed contract. The proposal review used profile 1.3 to assess the
clarification, not an exception to a MUST. Implementation status belongs to Plan 25m.

## Design-review recommendation audit

Use [design-review-audit](../../../../.codex/skills/design-review-audit/SKILL.md) for a deliberate
requested or explicitly planned reassessment, usually between a completed review and plan
creation (ADR-0213). Inherit its operator-selected focus and applicable standard. Reconsider
each recommendation and their interactions, and seek omitted improvements within that focus.
Judge diagnoses separately from remedies; retaining a recommendation is a valid result.

Publish a net-change companion at
`docs/design_review/reviews/audits/<principal-review-stem>_audit.md` and an integrated revised
review at the original principal path. The [folder guide](../../reviews/audits/README.md)
owns naming and repeat-audit conventions. The companion accounts for every input recommendation
with Retain, Improve, Correct or adjust, Omit, or Unresolved, plus additional recommendations and
their basis. Preserve the exact input in Git before replacement and cite its revision. Record
the current audit date, source/standard baselines and evidence limits, distinguishing changes
since the first assessment from corrections to it.

The revised principal explains the resulting design, with consistent judgments, recommendations,
dependencies and rule impacts. Briefly attribute the new judgment and link the companion; it
does not imply the original reviewer agreed. Keep critique of the first round in the companion.
Preserve finding/rule-impact identities and the difference between recommendation correspondence
and finding identity. Omitting a remedy can leave a valid diagnosis open. Existing test receipts
remain historical evidence at their original strength, date and applicability.

This normally continues the same engagement and does not itself require another broad audit;
reconcile changed premises and use the freshness rule if it becomes a new engagement. A fresh
`design-reviewer` can provide independent judgment, returning both texts for coordinator
publication. Recommendation dispositions are editorial, not execution status. Flag implications
for plans already consuming the review without silently changing accepted scope. Rule changes
still go to the operator in plan creation; the audit does not authorize remediation.

## Plan-completion audit

Use [plan-completion-audit](../../../../.codex/skills/plan-completion-audit/SKILL.md) at a
deliberate requested or explicitly planned claimed-completion boundary (ADR-0213), for the whole
plan or explicitly bounded scope. Reconstruct the accepted obligations from the intended behavior,
target design, authorized scope changes and acceptance conditions, including applicable consumer,
migration, preservation and retirement work. Inspect actual implementation beyond completed rows
or the reported diff. New design opportunities are distinct from unfinished accepted scope.

Update the same owning plan in place: a current Completion audit section records date, inspected
source state, claimed scope, bounded conclusion and evidence limits. Reconcile affected package
state, findings, completion claims and next work rather than appending a contradictory report.
The coordinator owns a series' combined conclusion; supporting plans retain affected local
details and source finding identities. Git holds prior accounts and receipts keep their original
dates and scope. No separate completion report, folder or progress register is introduced.

Distinguish functional implementation, required qualification and benefit measurements. Use
existing applicable evidence and targeted investigation as warranted; no automatic full test
campaign follows. Missing required evidence remains outstanding under the existing qualification
policy. A fresh `implementation-reviewer` can return the judgment and proposed plan corrections
read-only for coordinator publication. The audit identifies actionable repair scope without
authorizing production repairs, broad principle diagnosis or a new solution review. Neither new
audit is a recurring per-slice gate or an automatic part of every handoff.


## Architecture scenarios

These are reusable scenario seeds, not new product requirements or a demand to implement every
analysis mode. Select and refine those that distinguish alternatives in the requested scope.
A review may link a row and record only its case-specific conditions and acceptance.

| ID | Stimulus and conditions | Expected response / architectural observation |
|---|---|---|
| <a id="pse-s01"></a>PSE-S01 | Add a unit/property model using existing physical concepts | Declarations and specialized behavior extend existing owners; admission, diagnostics and workflow policy are not re-authored |
| <a id="pse-s02"></a>PSE-S02 | Replace/add an implementation of an existing consumed solver/provider capability | Integration and explicit selection absorb the change; semantic consumers retain their contract; differences in capability remain visible |
| <a id="pse-s03"></a>PSE-S03 | Compose a new study or analysis workflow | Reuse model preparation and execution primitives; identify genuinely new policy and avoid copied end-to-end flows |
| <a id="pse-s04"></a>PSE-S04 | Change a result storage/transport representation without changing scientific meaning | Persistence/conversion owners absorb the change; physical meaning and computation remain under their existing owners |
| <a id="pse-s05"></a>PSE-S05 | Exercise admission or execution policy independently | Tests supply the actual semantic inputs/dependencies without starting unrelated solvers, stores or workflows |
| <a id="pse-s06"></a>PSE-S06 | Upgrade an integration library within the intended capability contract | Identify integration owners absorbing API/lifecycle changes and any justified consumer contract migration |
| <a id="pse-s07"></a>PSE-S07 | Increase structure/case count, introduce skew or concurrent solves, or interrupt before publication | Necessary physical work, live state and queues remain credible for supported scope; prepared structure is reused, native work drains, recovery matches actual effects |

Inspect dependency edges from manifests and executing paths. Existing dependency/import checks
can support named boundary claims; an acceptance manifest is not an architectural dependency
map. No count of crates, traits, schemas or library features establishes quality.

## Follow-up ownership

Diagnostic findings retain stable DA-nnn identities; consuming reviews cite their audit snapshot.
New review-only findings use the review's existing convention without renumbering inherited
identities. Editorial recommendation dispositions are separate from implementation progress.
The revised review preserves the recoverable original and attributes the new judgment.
Plan-completion corrections update the scheduled owner in place.

The review records its version, inspected scope, findings and evidence strength. After a
finding is adopted for work, the owning plan's coverage/disposition table holds its current
state and links its scenario, decision where needed, packet and evidence. Until then the
review names a proposed work owner without claiming that work is scheduled. Use the template's
slot 11 fields; avoid an additional independently edited backlog.

The chain is foundation → scenario → finding → decision where required → packet → evidence.
Plan indexes link to the status owner rather than copying packet completion. A packet's
execution record owns its observations; a plan disposition links them. Accepted ADR evidence
records support at decision time, not live implementation qualification. Deferred architectural
decisions still use the existing ADR register and observable triggers. Workflow bookkeeping
and semantic judgments remain separate; there is no generated approval or architecture score.

## Where profile roles are decided

| Role (principle) | Authority in this repository |
|---|---|
| Architecture review and tracking (AP-01–AP-07, G9) | blueprint §24.4; ADR-0094/0162; the current core and template |
| Model authority and identity (DP-01, DP-04, DP-05) | blueprint §2 (D1–D14), §5 and §6; ADR-0088/0089 |
| Ownership of math, derivatives, sparse algebra, properties, structure and solvers (DP-13, DP-17, PS-07, PS-09) | blueprint §1.3, §3.2, §3.3, §7, §14 and §18; ADR-0082–0084 |
| Supported math, provider and solver scope (DP-15, PS-02, PS-10) | blueprint §9, §18 and §25; ADR-0082–0084 and ADR-0093 |
| Library capability evidence (DP-15) | `docs/capability-maps/` (`just lib-outline <file>` first) and optional overviews in docs/libraries/, Context7 and primary sources, plus the active native-solver-libraries skill |
| Error taxonomy (DP-21) | blueprint §23.2; every `pub enum *Error` derives `thiserror::Error` and implements `miette::Diagnostic` (AGENTS.md *Invariants*) |
| Dependency admission (DP-13) | [dependency policy](../../../dev/dependency-policy.md) and ADR-0066: no library or licence is refused in phases 0–1, so licence is never a reason to reject a §F candidate |
| Reference validation (PS-13) | IDAES parity against `idaes-pse==2.13.0` as a reference oracle, clean-room: read for behaviour, never copied ([relationship to IDAES](../../../relationship-to-idaes.md)) |

## Local policies that tighten the standard

Each is stated once in its authority; this list only points to it. They govern plan execution
and implementation review; a design review treats them as current design (Grounding), except
where one governs how the review itself is conducted or reported, such as evidence labels.

| Policy | Tightens | Authority |
|---|---|---|
| The baseline is zero: no lint finding, failing test or warning is tolerated at plan close | DP-23 | AGENTS.md prime directive 1 |
| Evidence labels (principles §D) are mandatory in ADR `evidence:` fields, PR descriptions and a plan's Verification and Outcome at close | DP-22 | AGENTS.md prime directive 4 |
| Never edit generated paths; fix the generator | DP-01 | AGENTS.md prime directive 2 |
| One resolved version per pinned dependency family | DP-15 | AGENTS.md *Invariants*; `just family-check` |
| `force_validate` in actual Arrow-consuming correctness closures; `panic = "unwind"`; never `target-cpu=native` | DP-03, DP-19, DP-11 | AGENTS.md *Invariants* |
| Delete replaced code, callers, tests and fixtures in the same change; no shims | DP-16 | AGENTS.md *Execution rhythm* |
| No library capability is withheld for lack of a consumer; integration cost remains assessable under the core | DP-13, DP-16 | blueprint §3.3.1 |
| Design alignment is established by agent judgment and review; probes, tests and written records are used where uncertainty warrants, not as proof of diligence, and there is no dedicated alignment tooling | DP-23 | this binding (maintainer decision, 2026-09-24) |

## Commands for evidence

| Evidence | Command |
|---|---|
| Targeted behaviour (*Tested*) | `just unit-package <pkg> <filter>` |
| Compilation of touched code | `just check-package <pkg>`, `just check` |
| Library fit where doubt remains (DP-15) | capability maps (`just lib-outline`), optional overviews, Context7/primary sources and the native solver skill |
| Dependency families (DP-15) | `just family-check` |
| Reference validation (PS-13) | `just parity` |
| Plan-close qualification | the [qualification command guide](../../../dev/validation-assessment.md) and AGENTS.md *Qualification reporting* |

## Routes for required changes

A review's rule impacts (`review#RCnn`) go to plan creation for operator confirmation first; an
accepted change then uses the route below, and a rejected one stays as the plan records it.

| Change | Route |
|---|---|
| Alters D1–D14, adds or removes a crate, governance, SHOULD deviation | ADR plus design review (AGENTS.md *When an ADR is required*) |
| Blueprint text | `design:` PR with a revision row (`PSE_DESIGN_EDIT=1`) |
| Deferred decision with a trigger | register row in `docs/adr/register.md` |
| Sequencing or scope | the active plan in `docs/plans/` |

## Known conflicts

| # | Standard says | Repository text says | Until resolved |
|---|---|---|---|
| K3 | Foundation IDs `AP-nn`, refinement IDs `DP-nn`, profile IDs `PS-nn`; evidence vocabulary in principles §D; exceptions in §H | Retained accepted ADRs cite `DM-nn`, "charter §D" and "charter §H" | Read them through principles §I; accepted records are not edited |

K1 (library-first mathematics versus pre-Plan 14 blueprint text) is resolved: the
architecture sections now describe library-owned mathematics, providers and native solvers.
K2 (a preference for runtime mechanisms over generated projections) is resolved: Core 3.1
dropped that preference, and generation from one authoritative declaration is judged on
its merits like any other mechanism.

## Where defect shapes tend to land in this stack

Illustrative places the shapes in the `design-review` skill's reference have landed before.
Their absence proves nothing.

| Pattern | Shape | Principles |
|---|---|---|
| An ordinary workflow extension edits unrelated policy, provider and presentation owners | Change amplification; trace the responsibility crossings | AP-01, AP-03 |
| Policy tests construct a full runtime that initializes native/storage state | Unnecessary test dependencies, if the policy needs only explicit inputs | AP-02, AP-06 |
| Consumer branching on incidental backend objects | Leaked implementation knowledge | AP-02 |
| `Schema` / `Field` built in more than one module for the same logical table | Second authority | DP-01 |
| `DataType` matched exhaustively in several places, each with its own coercions | Second authority, silent degradation | DP-01, DP-15 |
| Sentinel floats (`NaN`, `-1`, `f64::MAX`) where a typed status or null is the meaning | Silent degradation | DP-02 |
| `unwrap` / `expect` / `unwrap_or_default` at a deserialization or FFI boundary | Unguarded boundary | DP-03, DP-15 |
| UDF `signature()` / `return_type()` wider than the kernel handles; `supports_filters_pushdown` returning `Exact` for an approximate filter | Unbacked capability | DP-15, DP-08 |
| Plan or `ExecutionPlan` equality as a cache key without configuration or provider version | Incomplete reuse key | DP-09 |
| Schema metadata (units, domains, provenance) dropped by `project`, `cast` or conversion | Silent degradation | DP-15, DP-21 |
| Interior mutability inside a path documented as pure planning or inspection | Hidden effect | DP-18 |
| A tracked function reading an untracked field or an input through a non-tracked path (Salsa) | Handle mistaken for version | DP-09 |
| Interned or tracked IDs held across revisions; untracked struct fields changed between revisions (Salsa) | Unsound equality, incomplete reuse key | DP-04, DP-09 |
| `NodeIndex` persisted, or results indexed by `node_indices()` after removals (petgraph) | Index-space bridge | DP-04 |
| `UnGraph` / `into_undirected` on directed relationships; `GraphMap` where parallel edges carry meaning | Silent graph reinterpretation | DP-07 |
| Two majors of a graph library in `Cargo.lock` through a backend adapter | Version bridge | DP-15 |
| A whole-graph algorithm behind a DataFusion operator that claims a partitioning or exact pushdown | Global computation per partition | DP-08 |
| Evaluator, property or solver workspace shared across attempts or held inside a tracked query | Owned-workspace violation | DP-19, DP-18 |
| Per-case structure reconstruction, per-scalar bridge calls, or independently maximized case/native pools | Unjustified work/crossing/live-state amplification across the complete solve route | AP-07, DP-10, DP-20 |
| Solver return code mapped to success without the post-solve check | Status read as success | PS-10 |
