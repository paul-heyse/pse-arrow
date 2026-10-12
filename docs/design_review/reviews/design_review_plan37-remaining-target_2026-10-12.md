# Design review: Remaining Plan 37 scientific ownership, reduced replay and operational activation

## 1. Scope, evidence and decision

**Decision: Accept the proposed target design.** Its scientific ownership model, reduced-history replay contract and maintenance activation boundary fit the simulator's functional target. **Behavioral/semantic adequacy: pass at Proposed design level. Architectural fitness: satisfied at Proposed design level.** This acceptance opens the dependent implementation route; it establishes neither implemented behavior nor Q1 qualification.

| Field | Value |
|---|---|
| Tier / purpose / boundary | Design / target / bounded changes to scientific artifact ownership, retained study inputs, detailed-history release, frozen forward rollout and fresh reset |
| Reviewer | Independent delegated design reviewer `/root/p37_target_review_resumed`, 2026-10-12 |
| Standard | Core 3.4, Heuristics for Efficient Architecture H1–H28, Process Simulator 1.5 and pse-arrow binding selected by `standard.toml` |
| Source baseline | `d30599417e172594877ba94a7659e9e6c8ae8fbe`; production source unchanged by the reviewed proposal |
| Proposed subject | Working ADR-0182; blueprint revision 158 amendments to §20.3–§20.6 and §21; Plan 37a/A0 scientific contracts and Plan 37c C2/C3 |
| Disposition owner | `docs/plans/37-surrealdb-lifecycle-and-integration.md` |
| Intended publication | `docs/design_review/reviews/design_review_plan37-remaining-target_2026-10-12.md` |

The target serves interactive edit/re-solve, durable studies, dynamics and scientific analysis without coupling a small useful scientific product to an unrelated producer's complete retained output. It also permits explicit release of exploratory detail without making old execution identities appear unused. Operational evolution remains under one maintenance owner, with actual effect settlement before explicit activation.

This is an independent completion after the previous review session was interrupted. Tentative messages from that session were treated as leads, not acceptance. I inspected the decisive proposed contracts and current neighboring source myself. No tests, builds, numerical probes or services were run for this review. **Interface-checked** below denotes inspected current seams; target behavior and benefits remain **Proposed**.

Excluded are a repeated review of B1–B3, reopening I1–I7 adoption choices, full solver/model correctness, all release gates of other active plans, power-loss qualification, remote deployment, representative performance/capacity and physical disk recovery. These exclusions do not narrow the selected lifecycle contracts. Final assembled implementation acceptance remains Q1.

## 2. Responsibilities and composition

The proposal separates three decisions that the baseline conflates through whole-producer retention: what scientific content is sufficient, who has an independent continuing obligation to it, and whether its producer's execution history remains available.

| Responsibility | Hidden decision and consumed contract | State/effect owner and local test setup |
|---|---|---|
| Scientific closure | Exact selected content, coordinates, units, validity, assessment and reuse permission | Existing scientific/runtime owners; pure closure/compatibility tests plus actual producer fixtures where required |
| Canonical artifacts | Stage, activate, protect, attach/release obligations, retire and collect admitted content | Operations and registry-generated transitions; owned store fixtures for atomicity and races |
| Durable study orchestration | Resolve external input before activation; bind retained input to occurrences and reuse it on dispatch/recovery | Runtime consumes scientific admission and canonical artifact handles; actual study fixtures for restart |
| Reduced history | Explicit payload/history release, dominating owner fences and immutable operation ownership | Canonical execution/study/retention owners; replay and collection fixtures |
| Operational evolution | Exact owner, pending nonce, admitted frozen inputs, physical drain and settled effects | Existing service/maintenance owner with fixed schema settler; disposable owned service fixtures |
| Activation | Validate exact installed definitions, codecs and scientific obligations; publish target identity only under admitted authority | Canonical owner; direct fresh creation and maintenance activation share target interpretation |

No universal artifact service, solver-independent scientific ontology, second reset manager or per-kind lifecycle framework is required. A new scientific product adds its genuinely new sufficiency contract and generated representation; generic storage continues to consume admitted closure. A new start format changes its scientific payload/compatibility owner rather than the collector. Changes to beta CLI details stay within tooling integration; scientific consumers do not interpret its progress catalog.

## 3. Scientific and identity contracts

### Sufficient products

The selected copied-content realization has a useful physical purpose: it severs producer collection authority while retaining only admitted content. A digest verifies encoding, not scientific permission. Original scientific identity and original encoding digest remain separate from the new artifact occurrence and copied encoding digest.

| Product | Required interpretation preserved | Authority |
|---|---|---|
| Seed/start | Exact original solution/run/attempt/step, closed inventory, primal IEEE bits, optional solver start state, layout/preparation/profile/data/backend, reuse assessment and optional prediction permission | Original scientific admission and target operation compatibility |
| Final state | Actual endpoint time, coordinates, state/input/output/integral ordering, units/basis, mode, endpoint requirement/satisfaction and coverage | Dynamic producer and prepared model/case interpretation |
| Trajectory | Exact selected sample/symbol membership and times, units, required event/mode transitions, coverage and original assessment | Trajectory scientific owner |
| Local analysis | Quantity/matrix ordering, units/scales, objective/dual conventions, validity/conditioning/withholding, statistical assumptions and required upstream input | Local analysis scientific owner |
| Graph evidence | Complete selected graph endpoints and typed artifact block/row/field evidence locators; structural reachability is distinct from numerical permission | Existing graph analysis owner consuming independent artifact inputs |

**Physical semantics:** dimensions, unit/basis, reference/convention and property/model validity remain those of the original scientific owners. Promotion preserves this interpretation and refuses missing required context; it does not convert units, widen envelopes, turn withholding into zero or upgrade a partial outcome. The actual endpoint is not assumed to be the last requested sample. Final-state retention does not claim an integrator continuation checkpoint.

**Well-posedness and numerical stages:** this change introduces no formulation, evaluator, derivative, scaling, structural-analysis or solver stage. Existing prepared operation admission remains responsible for problem class, structural validity and solver capability. Retained starts remain declared execution inputs; newly computed states still require the original independent checks. No numerical-stage table or fresh model-conformance claim is warranted for this bounded target.

### Admission, obligations and retirement

A creation intent binds original source authority incarnation, nonce, finite original expiry and exact request. Staging holds producer protection outside the final transaction. Atomic activation registers the producer/source conflict authority, validates sufficient staged closure and installs independent roots and obligations. No numerical work occurs inside a DB transaction. Unknown creation retains its exact intent, staged content and protection until actual settlement.

Study, user/project, analysis, protected-read and audit obligations have separate owners. Releasing one cannot invalidate another owner's future protected acquisition. Producer retirement closes producer reads while retained artifacts continue under their own authority. Retirement excludes new owners only when retained obligations permit it; an existing retained borrower keeps its contracted future protected reads. Final collection closes remaining acquisition/creation under exact guards, settles delayed windows and actual grants, then removes owned content and indexes in bounded resumable pages. Escaped copied arrays retain allocation ownership; they do not retain permission to read future DB pages.

The selected durable-study contract includes **target-case scientific compatibility before activation**, not just producer availability or the reconstruction of a stored `Compatibility` object. Target preparation can be reused; the store does not independently decide compatibility. An explicit incompatible input refuses before an active reproducible study exists. Within-study deduplication preserves per-occurrence role/provenance. Continuation from a future predecessor is not falsely treated as an already available external input.

### Reduced replay

Permanent run/study fences precede mutations, early receipt returns, readiness repair and recovery. Existing opaque operation IDs retain server-issued immutable ownership locators, installed atomically with the original effect. Owner/fence validation and receipt readback must share their transactional consistency boundary. An operation-only lookup followed by a separate later retirement check is not the selected target.

Live or uncertain effects retain exact immutable comparisons. After settled authorized history release, old requests return retired/history-unavailable even if their supplied bytes differ. A compact locator classifies identity and truthful disposition; it cannot reconstruct a full successful closure or authorize redispatch. Missing detailed receipts do not establish nonexecution. Reusing a retired study ID with a different live run refuses. Source/grant authority incarnations and monotone closed-through floors survive while delayed original creation/acquisition can still address them.

This retains compact state growing with issued opaque identities. That is an explicit tradeoff for unchanged ID/hash meaning, rather than an unbacked constant-space promise.

## 4. Revealing scenarios and execution fit

| Scenario | Kind and expected response | Required implementation evidence |
|---|---|---|
| S01: Admit an external start, restart the study, retire producer | Composition; retain exact qualified capsule and compatibility before activation, then dispatch without reacquiring producer authority | Both commit orders; actual restart before dispatch; identical bits/permission and recorded occurrence provenance |
| S02: New final-state or local-analysis product | Domain extension; scientific owner supplies new sufficiency/interpretation, generic lifecycle remains reusable | Missing interpretation refused; original partiality/withholding preserved |
| S03: Retain trajectory/graph evidence, collect unrelated producer payload | Representation/lifecycle change; independent artifacts retain context and typed evidence resolution | Event/mode and non-requested endpoint fixture; graph locator resolves after producer collection |
| S04: Release history, replay old opaque IDs during every collection boundary | Policy/effect change; fences dominate receipt/absence and no resurrection or redispatch occurs | Altered requests, different run supplied for retired study ID, delayed creator, unknown effect and resumed collection |
| S05: Change generated schema and interrupt beta SQL | Mechanism evolution; maintenance owns exact frozen chain, drains authentic prior SQL/listener, settles known effects and explicitly activates | Complete and interrupted rollouts, competing writers, actual process death, failed repair and ambiguous-step refusal |
| S06: Clean reset and stale client | Operational composition; exact selected owner regenerates current target with fresh authority; old capabilities refuse | Wrong identity/unrelated borrower refusal; interruption at durable phases; rebuilt worker and installed Python journeys |

AP-07 is credible without a benchmark for this proposed scope. The normal route reads sufficient selected content, copies it once into independent bounded blocks, stages outside short visibility transactions and uses indexed exact ownership/cursor paths. A small seed need not decode a full completion after retention. Long trajectories can use ordered bounded selections without keeping unrelated producer output. Deduplication is scoped to exact study inputs and does not erase occurrence attribution.

Copying is selected over shared producer blocks because it removes an otherwise additional backing-block ownership/reclaimer protocol. Large repeated overlapping selections could make sharing preferable; measured overlap and a complete independent block-retention contract would reopen that choice. Many retained operations still require compact identity storage. Bounded pages limit individual effects, while exact indexes and cursor continuation are needed to avoid repeated scans of unrelated state. Small fixtures can check access shape and correctness, not representative throughput or scale.

Maintenance exclusion is justified by schema mutation and reset, not attached to ordinary scientific reads. Existing local-first authority is reused instead of importing a distributed lease framework. Failure repeats the necessary exact step or stays unknown; it does not blindly replay a data-changing chain. Value-only model edits retain the existing structural reuse contract. Artifact copying and startup compatibility do not authorize rebuilding structure per value change.

## 5. Maintenance settlement and library fit

The beta tool owns schema selection, frozen SQL, ordered plans and its progress/repair machinery. The existing service owner supplies the contract it cannot supply: application/tool writer exclusion, authentic child/listener lifetime, pending identity and exact recovery. The canonical owner supplies validation and activation. The proposal does not introduce a second SDK universe or require broad beta actor/seed/watch support.

For acknowledged schema steps, settlement checks original pending nonce, immutable inputs, exact step identities, stored acknowledgement and actual generated-definition postconditions. For an unacknowledged **known schema-only** step, exact postconditions can reconcile effects only after authentic prior SQL/service drain establishes that the old operation can no longer modify them. Unsupported or ambiguous steps remain unknown. Data steps additionally need the domain owner's atomic witness/postconditions; a matching final catalog alone is insufficient. CLI success, tool repair, lock expiry and physical drain are each distinct from SQL settlement.

Explicit activation verifies exact definitions/codecs/scientific obligations and settled predecessor before publishing the marker. Fresh activation mints issuing authority; compatible forward activation preserves it. Ordinary open validates and cannot repair or activate. The schema SHA used by beta does not replace the canonical digest/interpretation. Immutable producer-owned rollout steps remain derived from the registry; generation cannot silently rewrite published history.

Library-owned record references/cascades, engine VERSION and shared external files do not replace attributed scientific ownership or sufficient content. These were settled bounded investigations and are not reopened here. The simplest viable comparator is owned selected copies with indexed explicit obligations. Whole-producer pins retain unrelated output; shared blocks retain a more complex lifecycle; maintaining all detailed history avoids deletion work but defeats explicit detail release. The selected design adds only the domain/authority gap that those mechanisms do not discharge.

This review inspected existing integration seams rather than making new SDK/CLI API claims. Exact-release tooling qualification remains with the recorded C0/C1 evidence and future C2 controls; it is not inferred from current online documentation or repeated here.

## 6. Architectural foundations and gates

All verdicts concern the **Proposed target**, not the baseline implementation.

| Foundation | Verdict | Reason |
|---|---|---|
| AP-01 | Satisfied | Scientific sufficiency, durable storage ownership and operational activation have separate reasons for change and explicit consumers. |
| AP-02 | Satisfied | Artifacts preserve consumed scientific meaning; backend/tool catalog details remain within integration owners; incompatible target stores refuse. |
| AP-03 | Satisfied | Studies, graphs and operational recovery compose existing admission/storage/service capabilities instead of duplicating workflows. |
| AP-04 | Satisfied | Independent artifacts, obligations, history release, opaque ownership locators and unknown effects are adequate concepts with governed operations. |
| AP-05 | Satisfied | Atomic visibility, exact guards, permanent fences, creation floors and explicit activation are enforceable declared boundaries. |
| AP-06 | Satisfied | Closure/compatibility logic retains local semantic dependencies; effects use authentic store/service fixtures rather than hidden infrastructure in pure policy. |
| AP-07 | Satisfied | S01–S06 have credible selective, bounded and effect-matched physical routes; copied content and permanent compact locators have explicit tradeoffs. |

| Gate | Target verdict | Evidence/reason |
|---|---|---|
| G1 Authority | Pass | Scientific owners define meaning; registry derives representation; canonical/maintenance owners govern their distinct effects. |
| G2 Fidelity | Pass | Exact bits, identity, physical interpretation, permission, partiality and graph evidence are retained; missing sufficiency refuses. |
| G3 Validity | Pass | Compatibility precedes activation; closure/ownership/maintenance premises have explicit validation and refusal. |
| G4 Hidden behavior | Pass | Promotion, release, reset and activation are explicit effects; ordinary open is validation-only. |
| G5 Consistency/recovery | Pass | Short atomic activation and dominating fences; exact unknown effects remain protected until settled. |
| G6 Transformation/reuse | Pass | Copied encoding is distinct from scientific identity; original input attribution and value/structure reuse remain intact. |
| G7 Truthful claims | Pass | Proposed maturity and retry/checkpoint/physical-space limits are explicit; no CLI/drain result is promoted into SQL or scientific success. |
| G8 Library leverage | Pass | Existing storage/tooling capabilities and owners remain in use; bespoke obligations/settlement cover their actual semantic gaps. |
| G9 Architectural fitness | Pass | All applicable foundations satisfied for these bounded target changes. |
| PS-G1 Physical consistency | Pass for preservation boundary | Sufficient interpretation and original validity/assessment retained; no new balance/property formulation claimed. |
| PS-G2 Well-posedness | Not applicable | No new structural-analysis or solve-admission policy; existing prepared operation owner remains responsible. |
| PS-G3 Numerical integrity | Pass for retention/reuse boundary | Promotion cannot certify new states or turn partial outcomes into solutions; target-compatible starts and independent new-state checks remain required. |

## 7. Findings and preservation constraints

**No new material target findings.** The target addresses the decisive current seams inspected:

- `workflow/durable.rs::admit_seed` protects the original inventory, reconstructs stored compatibility and currently decodes the full completion. `workflow/study.rs` checks `with_admitted_start` later in candidate preparation. A1 must move the actual target-case check before activation while A2 retains sufficient content without the full completion.
- `canonical_execution.rs::settle_operation` currently returns `None` on absent receipt and uses a subsequent retained-run check; `resume_closed_attempt` starts from operation-only lookup. A3's compact immutable locators and same-transaction fences must replace these assumptions before detail collection.
- Generated study `create` checks an existing header and its supplied run; it has no independent retired-study identity check after header removal. A3 explicitly closes that future deletion hazard.
- `surreal_analyses.rs::begin` and `surreal_retention.rs::settle_analysis` already model authority/nonce/expiry creation and monotone floors. Artifact creation must preserve that exact delayed-creator protection rather than substitute an unowned renewable lease.
- `workflow/modeling/trajectory.rs` exports endpoint records separately from samples and modes. A2 preserves endpoint semantics and emits missing interpretation while preparation is present.
- `surrealkit_maintenance.py::settle` currently requires a supplied domain settler and keeps admission closed; C2 must provide the fixed production schema settler and separate canonical activation, rather than treating its callback shape as established domain proof.
- `surreal_server.py` currently rebuilds with preserved input authority. C3 intentionally replaces that reset precondition under OP01, while retaining current-version restore's distinct backup contract.

These are implementation obligations already selected by the target, not evidence that the proposed correction is implemented. Some remaining architecture clauses describe the baseline full-analysis/result-input pin contract; update them with the A2 caller transition. They do not select a competing new rule. Preserve supported scientific source/history semantics while deleting replaced substrate historical readers and backfill paths.

## 8. Verification, rule impacts and follow-up

**Checked:** selected standard/core/template/profile/binding; working ADR-0182 and revision 158 amendments; Plan 37a/A0 and C2/C3; decisive seed/target-compatibility, execution readback, study creation/readiness, analysis creation/floor, endpoint projection, maintenance settlement and rebuild source seams. This is static source/interface inspection. **Measured: none. Tested: none in this review.** No claim of full source census, compiled interface, assembled failure recovery or scientific model qualification follows.

The revealing implementation controls are S01–S06, including both commit orders, all receipt/absence/recovery families, restart between collection pages, multiple independent borrowers and audit/unknown retention. Scientific expected values and lifecycle expected outcomes need independent assertions. Final installed Rust/native/Python journeys must use rebuilt consumers and the final fresh schema. Full unit/property conformance is not changed or newly established here.

**New rule impacts: none.** The proposal applies already confirmed Plan 37 RC01–RC03 (retained inputs/artifacts/history), RC09 (tooling under canonical maintenance) and OP01 (fresh reset). ADR-0182/revision 158 record their changes to blueprint §20.3–§20.6 and §21 and to preservation/cutover provisions of proposed ADR-0172/0180. Keeping those older preservation provisions would obstruct the approved clean reset; keeping whole-producer-only ownership or full-history-only replay would prevent the selected retained-product/detail-release outcomes. No further operator decision is requested by this review.

Plan 37 remains the sole disposition and acceptance owner. Implementers should not turn the preservation constraints above into a second policy system or new review backlog. A newly exposed contradiction affecting validity, authority or feasible ordinary work requires reconciliation; unexecuted controls do not alone invalidate an otherwise coherent proposed target.

## 9. Final decision

**Accept the bounded target design at Proposed evidence level.** There is no identified MUST gap or unresolved material design premise in the inspected scope. The independent review does not accept the baseline as already satisfying the target and does not accept Q1 on the strength of documents.

The next consequential step is implementation of the shared artifact and reduced-replay contracts, preserving preactivation scientific compatibility and exact transactional ownership. Production maintenance settlement/activation and clean cutover then require actual failure controls before assembled acceptance. Current plan status, implementation progress and eventual qualification remain with Plan 37.
