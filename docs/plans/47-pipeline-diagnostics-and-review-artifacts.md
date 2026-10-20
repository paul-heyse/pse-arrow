---
title: "Plan47: Pipeline diagnostics and review artifacts"
status: complete
date: 2026-10-19
adrs: []
review_sources: [docs/design_review/reviews/design_review_pipeline-diagnostics-and-review-artifacts_2026-10-18.md]
scenario_sources: [docs/design_review/reviews/design_review_pipeline-diagnostics-and-review-artifacts_2026-10-18.md]
doc_role: plan
doc_owner: docs/plans/47-pipeline-diagnostics-and-review-artifacts.md
doc_topics: [design, diagnostics, numerical-execution, performance, documentation]
---

# Plan47: Pipeline diagnostics and review artifacts

## Purpose and basis

Deliver useful causal assessment of simulation execution while it runs and after completion, a composable CLI review workflow, and self-contained offline review packs for external assessment. The [source review](../design_review/reviews/design_review_pipeline-diagnostics-and-review-artifacts_2026-10-18.md) supplies F01–F03, RC01–RC03, nine revealing scenarios, a ten-context coverage boundary, the native capability catalogue, ten cross-owner questions and bounded further investigations. All are covered here; the three findings are not the whole scope.

**Authoring baseline (2026-10-18). Proposed:** the target and work packages in this series. **Interface-checked:** the current observation, interpretation, context/replay, protected result export and CLI foundations were inspected during authoring against HEAD `6524e8591` and the retained uncommitted review documents. At authoring, production source was unchanged from the review's `e9d24396c` basis. The review's passive-reader control is **Tested** under its original conditions; no new simulation, native qualification, observer-overhead measurement or product implementation was performed during plan creation. Accepted target decisions do not establish implementation acceptance.

The applicable standard is Core 3.4, Efficient Architecture Heuristics 1.0, Process Simulator 1.5 and the pse-arrow binding. The functional target remains edit/re-solve without structural rebuild for value changes; studies, recycles, dynamics, sensitivities, optimization and fitting; extensible scientific/native capabilities; and responsibilities testable with their actual dependencies. Blueprint §23.1 owns current diagnostic contracts; blueprint §§15–19 own numerical/workflow meaning. This series owns adopted review dispositions and investigation outcomes. Other plans retain their status and scope.

Creation of this series authorized documents only. Subsequent operator authorization covered A–F execution; the checkpoint and companion Outcomes own current progress. Other implementation plans retain their independent scope/status and the source review retains its historical assessment. Navigation, lifecycle bindings and current-work selection are maintained alongside this series.

## Reading route and ownership

| Companion | Owned target and packages | Consumed boundary |
| --- | --- | --- |
| [47a: Frozen evidence and review context](47a-frozen-evidence-and-review-context.md) | A01–A05; F02, descriptive context and selected relationships | Existing copied observation, scientific source and result identities |
| [47b: Native diagnostics and causal attribution](47b-native-diagnostics-and-causal-attribution.md) | B01–B06; existing signals and bounded native additions | Safe performing-owner observations; original coordinate meanings |
| [47c: Analysis and recording](47c-analysis-and-recording.md) | C01–C06; ten causal questions, selected recording, queries and profiling | Immutable evidence, qualified producer facts and explicit policy |
| [47d: Offline review-pack publication](47d-offline-review-pack-publication.md) | D01–D05; F01 and portable external interpretation | Frozen selection/context, original raw data, shared analysis and protected results |
| [47e: CLI review workflows](47e-cli-review-workflows.md) | E01–E05; F03 and cooperating collection | The same owned read/record/analyze/export operations |
| [47f: Qualification and design publication](47f-qualification-and-design-publication.md) | F01–F04 packages; selected assembled acceptance and enduring publication | Integrated functional delivery and settled investigation outcomes |

Package IDs are companion-local. For example, package 47f/F01 is distinct from review finding F01. Companions own package progress; the tables below own finding and investigation state. No index or report creates another editable task ledger.

## Combined target and foundation assessment

Scientific/model/native owners produce copied facts; pure interpretation and library queries consume them; CLI and offline publication present their results. This direction already exists and remains suitable. The source-inspected gap is the composition of recorded selection, time, context, membership and completion, rather than missing instrumentation everywhere.

| Foundation / current evidence | Resulting design choice |
| --- | --- |
| `inspection::inspect_image` uses current monotonic time and neighbouring checkpoint I/O; low `select_window` accepts an explicit endpoint | Add an explicit frozen route with supplied immutable members. Share pure window/interpretation helpers; keep live transport separate. No host/boot attestation is needed. |
| `inspection_comparison` already interprets borrowed invocation documents; typed Arrow evidence preserves source identities and availability | Consume these meanings from files as well as live objects. New question recipes reuse domain interpretation and library relational operations. Do not reproduce them in Python. |
| ReplayDocument exports substantial sources/settings/constructors but has stronger reconstruction premises | Add descriptive review context independently of executable capability admission. Share common source facts where appropriate; preserve scientific Replay's stronger contract. |
| Result IPC export already has exact protected lineage and incomplete staging; capture/detail have independent coverage and readable-prefix semantics | Compose these existing operations into selected pack membership and publication. Keep their distinct completion meanings instead of introducing a universal transaction service. |
| Selected native/evaluator owners already supply extensive work/progress facts | Consume them first. Add only qualified observations that answer a named unresolved question, at the actual loop/getter owner. No copied solver controller. |
| Existing CLI is a thin passive/capture surface; richer APIs are separate | Extend that CLI over shared operations, including immutable-document input. Avoid a second orchestration or query platform. |

The resulting interfaces express five operations: frozen recorded selection; descriptive context projection; source-linked analysis; selected review-pack export/readback; and cooperating preconstruction recording. These are behavioral boundaries, not a demand for five new services, crates or registries. New serialized products have an authoritative versioned declaration; generated views follow their existing registry/generator route. Runtime/workflow composes suppliers; the low diagnostic crate acquires no storage, solver or Python dependency.

A pack is initially an ordinary local directory with a machine-readable index, readable Markdown/local HTML and selected raw/typed members. Archives are optional transport conveniences, not a second required implementation. Sealed publication, selection completeness, evidence coverage, scientific completion and result usability remain independently inspectable. A sealed pack for failed science can be valid; missing required explanatory context prevents a self-contained claim for that selected question.

### Preservation and exclusions

- Original model/source identities, units, bases, reference states, guards, physical envelopes and independent scientific assessment remain authoritative. Engineering accuracy stays **0.001 where currently selected**; diagnostic thresholds, integration tolerances, derivative residuals, rank and KKT criteria retain their own meanings.
- Native accepted/trial checkpoints, original physical states, source sequence and durable append order differ. Generation/reset, sampled/lost/refused/nonfinite/overflow/unrequested evidence stays explicit. A source-map alternative is not an executed branch; native scaled coordinates are not automatically physical coordinates.
- Optional observation failure never changes scientific permission or stops unrelated execution. Native handles, charges and cancellation/drain remain with actual work owners; passive readers retain only copied evidence. No hidden numerical recomputation or foreign getter occurs on read.
- Keep live summary expiry/lookback, invocation v2's preparation exclusion, current managed grants, independent writer loss and protected result access. Do not enlarge ordinary retention to implement an archive or infer historical closure from current-attempt pages.
- **Environment/provenance proof is permanently eliminated.** No runnable reconstruction package, mandatory tool stack, diagnostic database, generic solver normalization, automatic winner selection, full wheel installation, broad long benchmark campaign or tolerance change is in scope. Recording actual known library/options/clock semantics is explanatory context, not an environment attestation.

## Rule decisions

**RC01–RC03 were explicitly accepted by the operator on 2026-10-18**, selecting “Accept all three” in the plan-creation preparation. No new rule impact was discovered during authoring. Acceptance selects the following target boundaries; it does not accept an unknown native/profiling winner or authorize product implementation.

| Item | Accepted target and consequence | Route before dependent implementation | Decision / date |
| --- | --- | --- | --- |
| <a id="rc01"></a>RC01 | Add selected offline membership, independent pack completion and frozen interpretation over supplied members; retain live diagnostics and scientific separation. F01/F02. | A01 specifies the contract delta for blueprint §23.1 and proposed ADR-0192/0196. Use the architecture revision/design route; introduce a short contract/relation ADR if its actual declaration requires one. | accepted / 2026-10-18 |
| <a id="rc02"></a>RC02 | Add descriptive model/run/settings context that can be exported despite missing executable factories/native witnesses; keep scientific Replay's stronger reconstruction admission. F01. | A01/A03 update the relevant blueprint §19.2/§23.1 contract through the design route before consumers depend on it. Accepted ADRs remain immutable; use proposed rationale or superseding records as applicable. | accepted / 2026-10-18 |
| <a id="rc03"></a>RC03 | Extend the CLI over shared selection/analysis/frozen/pack operations and cooperating recording, with honest attach/unsupported coverage. F03. | E01–E05 amend the operating route in the diagnostic guide alongside delivered commands. Ordinary command/tooling changes need no invented ADR; changed semantic contracts use A01's route first. | accepted / 2026-10-18 |

Architecture amendments and any required ADRs belong to authorized execution, not this document-creation task. A01 records the concrete affected contract before dependent changes; F04 publishes final enduring meaning and reconciles status. A consequential newly discovered rule change returns for explicit decision rather than being hidden inside an investigation outcome.

## Dependency route

First deliver A01's contract decisions and A02/A05's working frozen input boundary. A03/A04 supply descriptive context and exact related-member selection. These are prerequisites for the corresponding pack claims, not reasons to delay independent native or query work.

B01 and C01/C02 can consume existing source facts while A02 is integrated. B02–B05 investigate their specific additional observations; C03/C04 deliver recording/query composition independently where source contracts are ready. C05/I06 resolves only tool/interval/symbol questions that affect selected analyses. C06 unifies derived output for CLI and packs.

D01 declares selected scope against A03/A04; D02 exports actual members; D03 derives readable/typed analysis; D04 seals independent publication; D05 establishes offline-only readback. E01/E03 can expose working frozen/analysis APIs before full pack delivery; E02 consumes acknowledged recording; E04/E05 complete pack and joined CLI journeys. F01/F02 establish selected assembled behavior and proportionate overhead; F03 runs scope-end checks; F04 publishes/reconciles the delivered design.

An interface agreement is not a working prerequisite. Consumers migrate with their supplying change and targeted acceptance. Remove replaced caller-side live-clock/sibling-file interpretations in frozen routes, duplicated analysis and manual glue once replaced; retain the distinct valid live route. Shared files and generated declarations have one integrating writer even where packages are logically independent.

## Finding dispositions

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| F01 | Review S03–S05/S08 | resolved | 47a/A03–A04; 47d/D01–D05; 47f/F01 | Original context/raw/protected results and independent publication passed composed relocated readback; see 47f assembled acceptance. Unsupported/missing products retain explicit limits. |
| F02 | Review S04/S07/S08 | resolved | 47a/A02/A05; 47d/D05; 47f/F01 | Recorded interpretation and relocated CLI/native readback retain rows, coverage and original references independently of reader time, neighbours and services; see 47f assembled acceptance. |
| F03 | Review S01/S02/S06/S09 | resolved | 47c/C03/C06; 47e/E01–E05; 47f/F01 | Actual CLI cooperating native recording/handoff, protected selected export, retained comparison and relocated offline review passed with failure/unsupported controls; see 47f assembled acceptance. |
| CA01 | Identity-bearing context ingress | resolved | 47a/A05; 47c/C01 | Interpreted payload identity/scope and overlapping-context reconciliation reject contradictions; original retries/related scopes remain explicit. |
| CA02 | Related operations with independent process clocks | resolved | 47c/C01–C02 | Analysis records actual process-clock selections; explicit windows do not project onto unrelated clocks without alignment. |
| CA03 | Bounded/cancellable analysis preparation | resolved | 47c/C01 | Analysis preparation, retained leases and cancellation cover source sizing, comparison, indexing and bounded output before allocation. |
| CA04 | Existing native counters and nonadditive gauges | resolved | 47b/B01/B06; 47c/C02 | Known native populations and valid latest gauges preserve actual method, reset, generation and unit meanings. |
| CA05 | Canonical study dependency readiness | resolved | 47c/C02 | Actual canonical inventory run/attempt association, explicit selections, fences and gaps enter shared readiness answers. |
| CA06 | Physical/source attribution | resolved | 47c/C01–C02; I05 | Retained map ingress and supported findings/native fields resolve to original coordinates, occurrences, affine maps and source terms. |
| CA07 | Performed engine observations | resolved | 47c/C01/C06; 47d/D02; 47e/E03 | Actual performing-session receipts reach context, shared analysis and export, with bounded copy/lifetime and mirror suppression. |
| CA08 | Joined selected live/durable/scientific export | resolved | 47d/D01–D02; 47e/E04–E05 | Runtime and standalone CLI compose supplied handoff/live subjects and multiple exact protected field/range selections through shared export. |
| CA09 | Question-dependent explanatory closure | resolved | 47d/D01/D05 | Shared question/product prerequisites govern analysis and pack readback; missing required facts block self-containment. |
| CA10 | Typed diagnostic IPC in review packs | resolved | 47d/D02–D03/D05 | Registry diagnostic IPC has its own typed schema/source validation and publication/readback, independent from scientific lineage. |
| CA11 | Non-frozen source references | resolved | 47d/D01/D05 | Supported non-frozen statement and stored/recomputed prerequisite references resolve against actual supplied contents. |
| CA12 | Useful local human report | resolved | 47d/D03; 47e/E04–E05 | Local HTML links and bounded retained model/settings/table/profile orientation accompany useful Markdown reports. |

CA01–CA12 identify completion-audit gaps, distinct from the original review's F01–F03.
Their implemented corrections and focused acceptance are recorded in the
[corrective Outcome](#corrective-outcome--2026-10-18); the original audit below is historical.
Their implementation basis and closure conditions are recorded in the
[completion audit](#implementation-completeness-audit--2026-10-18). The original review
and accepted rule choices are unchanged; this table remains the single finding-disposition owner.

## Investigation outcomes

An investigation completes with an implemented qualified adoption or a justified no-adoption/support-limit decision and observable reopening premise. It cannot resolve an outstanding defect merely by declining a candidate. This table owns decisions; companions contain the actual question, source evidence, control and consequences.

| Inquiry | Decision question | State | Decision / work owner | Completion or next trigger |
| --- | --- | --- | --- | --- |
| I01 | Can IDAS local-error/weight/tolerance-scale observations safely identify relevant coordinates? | resolved | 47b/B02 | Adopted bounded state-only local-error observation after fresh eligible native returns. Four native controls qualify masking, WRMS, lifecycle, refusal and optional failure; see 47b Outcome. Failed/repeated/zero-step/reset observations remain unavailable, and no tolerance change follows. |
| I02 | Which extra FERAL snapshot fields explain a real factor-path change without fresh numerical work? | resolved | 47b/B03 | Adopted current scaling/order/front/perturbation and MC64 path facts through the existing single snapshot and factor event. Source/native controls qualify fatal clearing and no-new-factor state; see 47b Outcome. No O(1) or overhead claim. |
| I03 | Can the actual POUNCE loops expose inner retries/refinement with genuine parentage? | resolved | 47b/B04 | Reused existing actual refinement/RHS/residual populations, added actual call/group parentage and qualified narrow MC64 retry/fallback deltas. Other inner work remains unknown. Source and native controls are in 47b Outcome. |
| I04 | Does private Clarabel/Uno phase detail change a selected diagnosis enough to justify a library/bridge change? | resolved | 47b/B05 | No private hook adopted: no concrete unresolved phase question justified it after existing work/progress. Uno accepted/trial wording is corrected and copied-record controls passed. Reopen only when a selected opaque phase still blocks the next action after existing observations and a useful profile. |
| I05 | Which physical pullbacks are already valid, and where does selected analysis need more mapping or explicit numerical work? | resolved | 47a/A03; 47c/C01–C02; 47e/E03 | CA06 correction admits retained source maps and follows supported findings/native coordinates to original rows, terms, occurrences and affine coordinates. Focused producer/ingress/attribution controls passed; see corrective Outcome. Unknown/native dual or stationarity pullbacks remain explicit; no universal pullback or fresh numerical work is inferred. |
| I06 | Which profiler/interval/export/symbol capabilities actually serve the remaining question offline? | resolved | 47c/C04–C05 | Adopted running-max/union interval SQL and canonical unsigned text companions; actual Trace Processor 58.2 queries and hand-known totals passed. Retain perf/Samply with actual symbol/unwind/loss limits. Examined Samply import ignores its presymbolication setting, so no sidecar correction/additional format/platform adopted. Reopen for an unresolved concrete stack/allocation/off-CPU/phase question or a qualified import correction; see 47c Outcome/supporting inquiry. |
| I07 | Can selected failure/retry/many-child/context/profile/result exports close membership with credible work and live state? | resolved | 47a/A04; 47d/D01–D05; 47f/F02 | Selected 16/129-member growth/skew and nine fresh-process observer controls passed. Selected-source comparison avoids unrelated JSON trees; descriptive bounds exclude broader retained owners. Original IDAS segment counters remain available despite compact curtailment. See 47f measurements and assembled limits; no total-private-peak, complete-history or capacity SLA is claimed. |

## Complete recommendation coverage

This mapping is navigation, not another status table. References name rows/topics in the principal review rather than creating new review findings.

| Review material | Explicit treatment |
| --- | --- |
| §§1–3 target, responsibilities, contracts, physical semantics/well-posedness | Shared target/preservation above; A01–A05 and D01–D05. No new formulation or altered scientific acceptance. |
| §4 S01–S09 and all ten run-context rows | A03/A04 membership and context; E05/F01 context-specific propagation, including pre-handle, responses/covariance, initialization/recycles, dynamics/adjoints/shooting, fitting, studies/native children, horizons/background, surrogate/fidelity and later access. |
| §5 native catalogue: Ipopt; KINSOL/KLU; Diffsol/faer; HiGHS; SCIP; PETSc | B01/B06 consume current applicable signals and preserve method/stage/reset/activation semantics. C02 uses them; no compulsory new hooks or blanket requalification of unchanged libraries. |
| §5 POUNCE, FERAL, IDAS, Clarabel and Uno | B02–B05/I01–I04. Distinguish concrete safe candidate, conditional private-hook inquiry and Uno wording correction; no automatic tolerance/backend change. |
| §5 ten cross-owner questions | C02 owns all ten: edit rebuild; evaluator demand/reuse; cost per evaluation; root/recycle progress; dynamic/checkpoint work; fit/study readiness; factor recovery; waiting; comparability; physical-source attribution. |
| §5 operation hub; JSONL detail; timeline; engine observation | A02 keeps live/frozen meaning distinct; C01/C02 consume existing engine policy/plan/operator/lifecycle observations, C03/C04 preserve activation/interval/loss/query scope, and D02 exports selected copied reports. |
| §5 CPU/Samply; optional heap/off-CPU/native tools | C05/I06, including host-version/symbol/unwind limits and discovery-only Heaptrack/Callgrind/BPF/Tokio Console/SUNProfiler/Caliper/uProf. No mandatory installation/platform. |
| §5 Arrow/DataFusion/Trace Processor; explicit numerical diagnostics; offline analysis | C01–C06 shared projection/policy/query; D03 typed/readable output; E03 explicit additional work uses its existing origin/deadline, never passive read. |
| §5 overhead/execution fit; §§8–9 library fit and alternatives | B06/C03/I06/I07/F02; selective copy/stream/owner-safe observation, current libraries before new machinery. Directory pack first; Replay, profiler-only/manual exports and new platforms retain their stated different scope. |
| §6 foundation/gate judgments and profile preservation | A01–A05, D01–D05 and F01 assess the corrected composition; source inspection/short diagnostic tests do not certify underlying physical models or whole-product numerics. |
| §7 corrective contracts, challenges and verification | Finding table and supplying companions; priority follows fidelity/consequence, prerequisite order follows actual needed capability. |
| §10 all six further-investigation topics and evidence limits | I01–I07 and B/C/F packages; retain source evidence, candidate uncertainty, pre-handle/retry scope, Uno wording and next discriminating observations. |
| §11 RC01–RC03 and §12 integrated target | Accepted rule table, A01 route, dependency route and overall completion below. |
| Supporting native/library notes and reader control | Exact consumed releases/forks, IDAS local-error safety, FERAL A+Δ inertia, POUNCE unknown inner work, Perfetto58.2 overlap/export limits, typed integer/nonfinite fidelity, IPC stream/file differences, local publication and incomplete prefixes; consumed in A–F rather than a new proof campaign. |

## Verification and completion

**Proposed verification:** each package first compiles affected targets and runs revealing local tests. Pure frozen interpretation, membership, policy and sealing tests use copied fixtures without solver/store startup. New producer hooks receive focused native controls and scientific-output checks; existing unchanged counters need not trigger an all-solver campaign. F01 selects short joined propagation journeys across actual changed run contexts. F02 measures only unresolved observer/export cost questions, initially one complete observation per case and repetition only when it could change a decision.

After functional integration, F03 runs relevant integrated/native/component/Python checks and `just hygiene`, including full continuing/no-fail-fast Clippy for the affected default/no-default/native graphs and required manual governance/docs/native/powerset checks. Preserve force-validation in correctness closures and use the fast editable correctness build when needed. No wheels or exact performance target is inferred. All executed checks name mode, zero baseline, actual failures and exclusions.

Completion requires F01–F03 corrected in production and their composed acceptance established; all catalogue/analysis/context obligations have implemented treatment or a justified conditional investigation outcome; raw and scientific meaning survives external readback; no replaced mechanism remains; accepted contract routes and enduring documentation are published; and this coordinator links actual Outcomes. Sealed packs do not establish new scientific permission. Unsupported original-context or profiling capabilities remain explicit, not silently scheduled into unrelated plans.

## Implementation-completeness audit — 2026-10-18

**Interface-checked:** the coordinator and three fresh independent reviewers inspected all
27 packages in 47a–47e against production source at HEAD `6524e8591` plus the current
modified/untracked implementation. Requirements were assessed independently of prior Outcome
claims. Fourteen packages align with their local targets; thirteen have incomplete functional
or consumer integration and are reopened in their owning package tables. The companion audit
assessments account for every package. C06 and E03 have working local adapters; their reopened
state concerns the promised composed answers/suppliers, not a second interpretation defect.

The audit followed frozen/context/inventory suppliers, actual native producers, shared
analysis and query consumers, recording ownership, protected result export, pack publication
and Python/CLI routes. Source inspection is the primary basis below. Existing tests were read
for what they exercise; earlier passes are retained, not promoted into complete implementation.
No new design review, repair, simulation, library adoption or 47f qualification was performed.

### Confirmed gaps and closure conditions

**CA01 — Interpreted context identity admission (P1).**
`workflow/review_context.rs::ReviewContextDocument::validate` admits known fact payloads as
syntactic JSON without reconciling identity-bearing decoded values. `FrozenEvidenceSet::validate`
checks contexts separately without rejecting duplicate/conflicting selected context identities.
The physical consumer can therefore establish coordinates from a different run under the selected
context header. Reconcile interpreted run/step/source identities with their actual enclosing scope
and supplied relationships before interpretation. Explicitly linked child contexts remain valid;
opaque descriptions need no executable admission. Closure includes a mismatched coordinates
payload, conflicting same-scope contexts, and a valid explicitly related child.

**CA02 — Explicit windows across independent clocks (P1).**
`review_analysis.rs::analyze` expands related operations across processes, while `events` applies
the same numeric explicit interval/endpoint to each source clock. This differs from the correct
independent-clock branch in `frozen_inspection.rs`. Parent-clock numbers can select or exclude an
unrelated child population. Use actual justified mappings or separate source-clock selections
with alignment uncertainty. A two-process fixture with deliberately displaced clock origins
must retain correct source populations; absent alignment must not imply a common timeline.

**CA03 — Analysis working state outside admission/cancellation (P2).**
`review_analysis.rs::events` builds a complete event map, serializes duplicate records and sorts
before the Builder's source-item allowance takes effect. `analyze` takes no allocation or
cancellation context; the Python binding only detaches it. File-size and final Arrow projection
limits do not cover that preparation. Bring selection/duplicate/sort work under the existing
admitted cancellable operation, select early while preserving required predecessors, and expose
finite refusal independently of science. This is a source-established missing mechanism, not a
measured slowdown. Closure exercises cancellation and refusal during preparation, not just output.

**CA04 — Available native facts skipped or assigned the wrong population (P1).**
`review_analysis/questions.rs::metric_question` and `family_question` omit relevant existing
producer populations, including Diffsol setup-from-checkpoint/error-test counters and KLU
conditioning-rejected recovery. The operation consumer always selects `metric.cumulative`,
although `MetricSnapshot` assigns nonadditive observations to `latest`. Actual IDAS Newton-shift
and checkpoint gauges consequently become unavailable in derived answers despite valid source
values. Consume the known applicable catalogue with units, generation/reset/segment and
latest-versus-additive meanings. Test real producer-shaped copied checkpoints, including live
scalars without a final segment summary; arbitrary unknown metrics need no invented classifier.

**CA05 — Canonical inventories cannot enter readiness answers (P1).**
`workflow/study_diagnostics.rs::StudyHandle::review_inventory` emits canonical occurrence/attempt
members with `operation: None`. `questions::inventories` discards inventories unless a member has
a selected operation ID. Real canonical inventory facts and gaps therefore disappear even when
run/attempt lineage can associate them with supplied invocation evidence. Join through supported
exact identities or explicit selection, retain unresolved links as gaps, and test actual-shaped
current/retained pages. Do not fabricate operation IDs or infer links from names/processes.

**CA06 — Physical attribution stops at coordinate counts (P1).**
The physical question counts sources and coordinate vectors rather than following original
findings/event fields through supported mappings. Existing native attempt variables/rows,
`AttemptStepSummary.findings` and `modeling/diagnostic_source_map.rs` supply material not consumed
by that answer. Frozen input has no source-map member/codec route. Add retained map ingress and
join supported findings to original variables/rows/occurrences, affine scales and source terms,
with valid references. Unknown eliminated/native dual/stationarity mappings stay unknown; no
universal pullback or fresh numerical recomputation is required. This reopens I05's integration
claim, while preserving its separately delivered explicit diagnostic-work boundary.

**CA07 — Actual engine observations do not reach workflow review (P2).**
`EngineSession::copy_execution_observations` works for a direct Rust session holder, but production
workflow/Python suppliers do not call it for sessions that actually performed the selected work.
The runtime session factory is not retrieval of those performed sessions. Supply their retained
copied observations through the public context/export path and shared consumers, preserving
Off/Contract/Diagnostic, partial consumption and drop scope. A composed workflow control must
consume actual performed-session facts without enabling a policy or rerunning the query.

**CA08 — Selected export remains split across disconnected APIs (P1).**
`python/pse/diagnostics.py::_subject` admits frozen files/packs; CLI export invokes only generic
file/frozen/analysis publication. `review_pack::copy_prefix` rejects Result members. The separate
`StoredResult.export_review_pack` exports one canonical relation/range and seals immediately;
it does not expose the Rust selected-result route's multiple/filtered selections. Compose exact
available live/durable selection, original descriptive context, protected results and requested
field/range/member selection through their actual owners into CLI export. Do not infer missing
live capabilities. Closure requires a CLI-only joined journey with original context and selected
scientific rows, exact lineage and borrower release—not another diagnostic-only pack fixture.

**CA09 — Self-containment ignores question-required premises (P1).**
`review_pack::add_frozen` records declared missing context and blanket supplied unavailable
fields; `seal` checks those declarations and required members, not selected-question dependencies.
An operation-only pack asking physical/comparability questions can claim self-containment with
no original context. Conversely, unrelated unavailable fields can block a narrower question.
Derive explanatory closure from selected questions/products and exact subjects, retaining
explicit missing premises without disabling unrelated facts. Closure contrasts missing required
physical context with unrelated optional omissions; export success remains a separate state.

**CA10 — Diagnostic IPC is validated as scientific IPC (P2).**
CLI `query --table` writes registry diagnostic IPC, but `review_pack::validate_encoding`
requires canonical scientific-result lineage for every Arrow IPC stream. A truthful generic
Query/Engine/Detail IPC member therefore cannot complete offline readback. Support its actual
registry/schema/source contract separately from protected scientific-result lineage, compose
publication, and check sealing/readback agree. Keep real scientific lineage and protection intact.

**CA11 — Non-frozen pointers need not resolve (P1).**
Selection validation checks a source-reference member exists and its pointer syntax, while
`validate_analysis_sources` resolves content pointers only for member `frozen`. A required Query
or Context reference can point at `/absent` in an available member and survive seal/readback.
Resolve supported references against actual supplied codecs and identities; unsupported or
unavailable interpretation must remain explicit and cannot establish required closure. This
is semantic consistency, not a request for authentication, hashes or environment proof.

**CA12 — Human report lacks substantive orientation/navigation (P2).**
`review_pack::finalize_reports` escapes the entire Markdown report inside HTML `<pre>`; statement
source links remain literal Markdown instead of clickable local HTML links. Reports provide
counts but not useful retained model/input/effective-settings orientation, selected table views
or profile findings. Even decoded retained symbol names are omitted. Render bounded useful
summaries and real local references using the shared analysis and retained authorities. Keep
raw profiles and symbol/unwind/loss limits. No new profiler platform or universal chart is needed.

### Probe and coverage limits

**Tested — passive public-API control, baseline zero assertion failures:** one bounded Python
control ran through `scripts/pse-env --native --resource-class compile` using the existing fast
dev correctness extension and generated copied diagnostic fixture. With ten selected questions
and zero contexts, export/readback reported `self_contained=true`; the physical answer retained
only its mapping limitation. A second pack with an available typed Query member and required
`/absent` reference also sealed/read back with `self_contained=true`. These observations confirm
CA09 and CA11; assertions checked the observed audit reproductions, not desired corrected behavior.
An initial probe-construction attempt omitted a required constructor argument and failed before
export; correcting that probe setup yielded the observations above. HTML's escaped Markdown
behavior was source-inspected. No solver, database, live capture or scientific result was created.
Temporary probe output is outside the repository; no evidence dossier is required.

The other findings are **Interface-checked** from production suppliers and consumers, not newly
runtime-qualified. Audit depth covered all package contracts and decisive implementation paths,
not every native backend execution, all scientific models, large/skewed selections, observer
cost, long simulations or full static/integration qualification. Those latter omissions are not
additional implementation defects. No accuracy/default change or environment/provenance work
is introduced.

## Current checkpoint

**Complete — 2026-10-19:** all 31 packages across 47a–47f are complete. CA01–CA12 and
original findings F01–F03 are resolved in production and their composed acceptance. [47f](47f-qualification-and-design-publication.md#assembled-acceptance-boundaries--2026-10-19)
records 127 native controls, 26 Python acceptance tests, nine fresh-process observer cases,
actual managed/current inventory and live native-reader acceptance, original-source comparisons
and retained limitations. I01–I07 have adoption/no-adoption/support-limit outcomes. Earlier
audit/repair checkpoints retain their historical conditions; they do not reopen repaired scope.

RC01–RC03 remain accepted. Blueprint revision 215 and proposed ADR-0196 record original
optional standalone context, retained dynamic segment statistics, bounded descriptive source
copying and selected-source table comparison. Numerical defaults and practical accuracies are
unchanged. The POUNCE dependency remains `7f5e2e9d78555bd53f780c1af186c8fd22c922f5`.
Both owned ordinary/reference services drained and stopped; data and evidence are preserved.

**Tested, baseline zero active failures:** [47f's Outcome](47f-qualification-and-design-publication.md#outcome)
records full continuing hygiene coverage with all seven initially failed leaves repaired, four
clean continuing Clippy modes, 419 depth-two feature combinations and 33 no-default package
checks, 1152 setup tests, 25 governance tests and 48 doctests. Original failed aggregate exits
remain recorded; repaired leaves supply composite acceptance. Documentation, native scope
bindings and current-work selection are reconciled. Integrated worker worktrees/branches are
removed, only main remains and no owned test service is active. No Plan47 implementation or
qualification work remains. The highest coordinator and consumed companions remain retained
under lifecycle preflight; other plans keep their independent scope. Eliminated environment/
provenance proof is not revived.

## Corrective Outcome — 2026-10-18

**Implemented:** CA01–CA12 now compose through production suppliers and public consumers:
identity-bearing context and source maps; independent-clock selections; cancellable admitted
analysis; actual native populations, canonical inventory and performed engine-session copies;
original-context producer handoff; selected protected scientific and typed diagnostic exports;
question-specific explanatory closure; source-reference validation; and usable local HTML and
Markdown reports. The implementation replaces disconnected export/rendering glue with shared
owners. Actual session receipts are bounded before cloning and reset before reused admission;
retained scopes preserve original clocks and operation identities without mirror double counting.

Analysis and pack envelopes are v2. Recording request/ACK and launch-result envelopes are v2;
detail and launch-request envelopes retain v1. Generators own the corresponding schemas,
Python contracts and extension stubs. Blueprint §23.1, proposed ADR-0196 and the diagnostics
operator guide carry enduring behavior; the companion corrective sections detail responsibilities.
All three owned worktree contributions were integrated into the existing checkout and reviewed
with their consumers; root integration corrections are authoritative. Prior dirty work is preserved.

**Tested, baseline zero failures, native solver/KLU/isolation/Uno/PETSc environment where
required, with explicit force-validation for Arrow correctness:**

- `just unit-package pse-runtime '<repair selection>' --features native-solvers,canonical-tests`:
  **84 passed**, 1,287 outside selection. The selection was `test(review_analysis::) |
  test(review_pack::) | test(frozen_inspection::) | test(review_context::) |
  test(handoff::tests::) | test(source_recording) | test(retained_source_map) |
  test(canonical_output_indexes_read_original_rows_and_skip_unrelated_blocks) |
  test(canonical_connected_results_reopens_declared_arrow_exact_range_and_cancellation) |
  test(kernel_source_map_preserves_shared_occurrences)`. It covers supplying facts, refusal,
  cancellation, selected protected rows, closure, exact typed IPC, source links and offline readback.
- `just unit-package pse-engine 'test(bounded_copy) | test(selected_copy) | test(cold_multirelation)' -p pse-admission`:
  **4 passed**, 208 outside selection; performed-session copy, bounds and retained lifetime controls.
- `python -m scripts.python_tests --unit-only python/pse/tests/test_review_workflows.py python/pse/tests/test_frozen_review.py -n 0 -q`:
  **18 passed**, one component deselected, against the refreshed native correctness extension.
  Includes original-context engine analysis and typed query IPC publication/readback.
- `python -m scripts.python_tests python/pse/tests/test_review_workflows.py -k test_cli_handoff_exports_multiple_protected_scientific_selections -n 0 -q`:
  **1 passed**, 14 deselected. The actual standalone CLI uses explicit deployment settings,
  joins producer handoff, exports multiple exact protected selections and reads a relocated,
  self-contained pack. Its isolated current-profile store was stopped through the registered
  drained route. The shared legacy profile was not migrated, reset or stopped.
- `just check-package pse-runtime -p pse-py` and its native/canonical variant
  `--features pse-runtime/native-solvers,pse-runtime/canonical-tests,pse-py/native-solvers` passed;
  `just check-package pse-benches --features native-process` passed. Subsequent focused builds
  exercised the final private analysis changes. `just codegen` passed; the fast editable
  `just py-sync-native-correctness` build passed. No full wheel installation was needed.

**Mistakes corrected:** independent inspection exposed receipt reuse, premature unbounded
copies, contradictory source lineage, unused fixed-formal rejection, handoff selection/aggregate
limits, lost entered recording scope and drift between analysis and prerequisite selection.
These were repaired with focused controls. Initial checks also found stale test callers/fixtures
and an in-process CLI test that incorrectly reused an active process budget; the test now executes
the real standalone command and selects the actual output partition. Environment-shaped setup
failures were resolved in an owned current-profile fixture, without changing another owner's store.

**Deliberate boundaries:** these tests establish the repaired mechanisms and selected composed
journey, not every scientific context/backend or large/skewed export. No performance improvement
is claimed as Measured. 47f retains broad assembled qualification, observer/export cost,
full static checks and final publication/overall closure. Missing/unsupported scientific or
profiling premises remain explicit, and copying artifacts grants no new scientific authority.

**Tested documentation/upkeep, baseline zero failures:** lifecycle `validate` passed with
758 native scope items and no errors. The coordinator's `scope --state open` view contains
only review F01–F03 and I07; all twelve CA items are resolved. `just docs` published 513
searchable chapters. Root `just turn-end` completed ADR-index and formatting successfully.
Its nonblocking Ruff step reported **201 diagnostics against the zero target** before the
formatter; this is not a clean lint claim. Those emitted diagnostics were subsequently corrected by the maintainer-authorized
[static repair](47f-qualification-and-design-publication.md#static-repair-outcome--2026-10-18);
remaining F03 qualification stays open. The three integrated owned worktrees and temporary branches were removed;
no owned test/build process or service remains running. The preexisting design-edit marker
was preserved after removing only this task's authorization note.
