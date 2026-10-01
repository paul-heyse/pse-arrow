# Agent coordination governance review — 2026-09-30

## 1. Scope, drivers and coverage

| Field | Assessment |
|---|---|
| Subject | PSE agent coordination policy, common/assigned role contracts, native Codex and Claude adapters, local process-skill consumers, instruction routing and decision/navigation owners |
| Standard | Core/template 3.3; process-simulator 1.3; `binding/pse-arrow.md` |
| Tier / purpose | Design / target, bounded to development governance |
| Reviewer | Independent delegated design reviewer, 2026-09-30 |
| Baseline | HEAD `35ed87ee901fc3b43e08bfa61a37505afef8851d` plus the uncommitted coordination changes and corrected binding qualification pointer |
| Decision | Behavioral/semantic adequacy: satisfied at static policy level. Architectural fitness: satisfied. Overall: **Accept** |
| Decision owner | ADR-0139 and blueprint §24.4; existing plans retain production work and finding disposition |

The functional target is proportionate delegation: use independent coverage, context isolation, capabilities or judgment when their benefit justifies handoff and integration cost. Small or tightly coupled tasks remain executable by the coordinator. The coordinator retains design, integration and acceptance; executors retain local discretion within their assignments.

Inspected sources include `.agents/roles/`, the six native definitions for each runtime, canonical `.codex/skills` process consumers, AGENTS.md, ADR-0139, blueprint §24.4, the blueprint revision row, documentation navigation, the qualification guide and the repository binding.

Dirty scientific/production work, production qualification, hooks and their checks, model effectiveness, cost savings and actual native-agent task execution are excluded. No simulator analysis mode, physical formulation or solver behavior changes in this scope.

## 2. Decomposition, ownership and dependencies

| Responsibility | Owner and consumed boundary |
|---|---|
| Task decomposition, shared ownership and acceptance | Coordinator; `.agents/roles/README.md:3–6,37–56` |
| Assignment effects, preservation and evidence return | Common worker contract; `.agents/roles/worker.md:3–26` |
| Mapping, library research, execution, correctness review and testing | Assigned role contracts, consumed by thin native adapters |
| Native models, effort, tools and sandbox defaults | `.codex/agents/` and `.claude/agents/`, maintained separately |
| Workflow preparation and action | Canonical `.codex/skills`; runtime aliases expose the same sources |
| Review cadence, decision authority and qualification timing | Existing binding, ADR process and AGENTS.md |

The division follows reasons for change. Runtime settings can change without reauthoring each workflow. Role behavior remains shared, while the coordinator composes useful contributions rather than invoking a fixed pipeline.

The native adapters explicitly load the common and assigned contracts. Removing duplicated Claude implementer instructions preserves their substantive constraints through AGENTS.md and the shared executor contract.

## 3. Contracts, authority and constraints

The governance model distinguishes assignment, baseline, effects, dependencies, sibling ownership, evidence, uncertainty, focused advice, formal judgment and final acceptance. These distinctions govern the corresponding instructions rather than merely describing return formats.

Briefs identify settled decisions, relevant dirty changes, permitted effects, sibling ownership, dependencies and completion evidence (`.agents/roles/README.md:37–46`; `.codex/skills/execute-plan/SKILL.md:24–30`). Workers use relevant authorities without repeating root orientation; necessary read-only investigation can extend beyond named files. Permission, preservation and test rules remain applicable.

Consequential negatives retain search coverage and limitations. Conflicting evidence, unsupported version transfer, unresolved ownership/contract decisions and a second failed repair trigger coordinator reassessment, with several possible remedies (`.agents/roles/README.md:50–69`). These triggers do not prescribe automatic reruns.

Canonical skills remain in `.codex/skills`; `.claude/skills` and `.agents/skills` remain aliases. Native adapters retain independent ownership. AGENTS.md preserves generated-path protection, scientific constraints, force-validation, memory limits, pinned commands, concurrent-work protection and qualification timing.

Physical-semantics and numerical-stage tables are not applicable: the reviewed operations allocate development work and interpret evidence; they do not formulate or solve physical models.

## 4. Representative change scenarios

| Scenario | Observation |
|---|---|
| Small, tightly coupled repair | The coordinator can complete it directly; no minimum role count or mandatory chain applies |
| Independent evidence investigation | The brief and task-relevant owners supply context; the worker returns attributed evidence and uncertainty |
| Coupled implementation | Shared contracts precede dependent edits; sibling ownership and integration dependencies are explicit |
| Consequential negative or second failed repair | The coordinator reassesses the question and chooses a remedy; a stronger worker is an option |
| Stronger evidence work | Codex routing uses a fresh built-in default role with explicit settings and the evidence contract; Claude routing distinguishes model override from configured effort |
| Independent review | Fresh context, concrete criteria, consumers, failure cases and an identified baseline support independent judgment |
| Navigation material moves | AGENTS.md retains operative constraints while existing reference owners receive the map and qualification table |

A governance-policy extension and runtime-adapter substitution are the relevant variation axes. Scientific domain extensions are outside this bounded review.

## 5. Mechanisms and execution

The policy uses existing native roles, shared Markdown contracts, process skills and decision routes. It adds no execution framework, telemetry, worker cap, calibration campaign or standing review requirement.

Configuration changes are explicitly distinguished from changes to already-running agents. Native sandbox defaults supplement the assignment contract; they do not establish that inherited live permissions are read-only.

Formal reviewers return complete reports for coordinator publication. Focused advice supports plan creation but cannot replace a formal review required by the binding.

## 6. Architectural assessment and gates

| Foundation | Verdict | Evidence |
|---|---|---|
| AP-01 Separation of concerns | satisfied | Shared behavior, native settings, workflow composition and acceptance retain distinct owners |
| AP-02 Stable contracts | satisfied | Assignment and return obligations are explicit; runtime limitations and substitutions remain visible |
| AP-03 Composition | satisfied | Optional capabilities compose under the coordinator; no fixed role sequence |
| AP-04 Domain model and authority | satisfied | The model captures consequential governance distinctions and governs role/adaptor/workflow instructions |
| AP-05 Explicit structure | satisfied | Baseline, effects, ownership, dependencies, escalation and independent-review conditions are declared |
| AP-06 Local reasoning/testability | satisfied | Workers use bounded task context; static policy inspection requires no unrelated scientific runtime |

| Gate | Verdict | Evidence or scope reason |
|---|---|---|
| G1 Authority | pass | Shared contracts, native settings and existing disposition owners have coherent scopes |
| G2 Semantic fidelity | pass | Relevant uncertainty, permission and review distinctions survive the revision |
| G3 Validity | pass | Operative protections and timing constraints remain; instructions fit the examined document budget |
| G4 Hidden behavior | pass | Effects and inherited-permission limitations are explicit |
| G5 Consistency/recovery | pass | Stable baselines, integration ownership and reassessment govern concurrent work and repair |
| G6 Transformation/reuse | pass | Context reuse and skill aliases preserve declared obligations; evidence is revisited after material integration |
| G7 Truthful claims | pass | Configuration existence is Implemented; quality and resource improvements remain unmeasured |
| G8 Library leverage | pass | Existing native role/configuration mechanisms are used without a bespoke orchestration framework |
| G9 Architectural fitness | pass | All applicable foundations are satisfied for the representative governance changes |
| PS-G1–PS-G3 | not applicable | No physical, well-posedness or numerical behavior is changed or qualified |

## 7. Findings

No material open findings in the final PSE scope.

An adjacent navigation defect encountered during review was corrected: the binding still referred to the removed AGENTS.md “Verifying work” table. `binding/pse-arrow.md:107` now points to `docs/dev/validation-assessment.md` and AGENTS.md “Qualification reporting”. The reviewer inspected the corrected source and relative target on 2026-09-30.

## 8. Library/runtime fit and ownership cost

Existing native custom roles provide the required responsibility and runtime-setting boundaries. Shared contracts avoid duplicating behavior in every adapter. Runtime-specific stronger-worker instructions expose actual differences rather than inventing a uniform override interface.

The remaining maintenance cost is keeping independently maintained native settings consistent with the role table. That cost is explicit and bounded; a new generator or higher-tier role family is not justified by this change.

## 9. Alternatives and tradeoffs

Broad parallelism preserves familiar instructions but leaves handoff and context costs insufficiently governed. A fixed role pipeline, automatic stronger-model reruns or calibration campaign would add obligations without a demonstrated consumer.

Selective delegation is the simplest viable alternative serving the approved target. It retains coordinator judgment and independent assessment where consequential. Stronger defaults may consume more resources; this review establishes no measured quality or cost advantage.

## 10. Verification

| Claim | Evidence | Result and limit |
|---|---|---|
| Policy and consumer coherence | Independent source/diff inspection using `cat`, `sed`, `nl` and bounded `rg` searches | **passed**, 2026-09-30, for the named governance sources |
| Canonical skills and aliases | Source inspection and `ls -l .codex/skills .claude/skills .agents/skills` | **passed**; PSE canonical `.codex/skills` ownership preserved |
| Cumulative instruction size | Reviewer Python filesystem inspection | **passed**: global 2,691 bytes plus PSE 27,658 bytes = 30,349 bytes, below 32,768; `/AGENTS.md`, `/home/AGENTS.md` and `/home/paul/AGENTS.md` absent |
| Native setting consistency and configuration/skill discovery | Coordinator receipt: `python3 /home/paul/.cache/coordination-acceptance-20260930.py`; reviewer inspected script | **passed** according to the coordinator receipt; establishes settings/discovery, not task effectiveness |
| Corrected qualification pointer | Independent source and target inspection | **passed**, 2026-09-30 |
| Product qualification, hygiene, model-quality trials, benchmarks and native role task execution | Outside the assignment | **not_run** |

Static evidence is sufficient for this bounded governance judgment. These observations do not establish better model behavior, lower cost or scientific qualification.

## 11. Authority changes and disposition

ADR-0139 supplies the replacement rationale, with blueprint §24.4 and its revision row amended together. ADR-0138 remains accepted while ADR-0139 is proposed pending this review. Acceptance/supersession finalization follows the existing ADR route.

No SHOULD exception or MUST gap is required. Existing plans remain the owners of production work and scheduled findings. This review creates no recurring review requirement, second ledger or new acceptance machinery.

## 12. Decision

**Accept** the PSE coordination governance change at its static/Implemented policy evidence level.

Behavioral/semantic adequacy and architectural fitness are independently satisfied within this boundary. The sources preserve local scientific and operational protections, establish proportionate delegation and uncertainty handling, and retain meaningful independent review.

This acceptance does not certify the enclosing simulator architecture, dirty production work, runtime effectiveness, model quality or release qualification.
