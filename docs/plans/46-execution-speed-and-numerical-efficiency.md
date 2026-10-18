---
title: "Plan46: Execution speed and numerical efficiency"
status: done
date: 2026-10-18
adrs: [ADR-0145, ADR-0166, ADR-0180, ADR-0185, ADR-0186, ADR-0193]
review_sources: [docs/design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md, docs/pse-arrow-execution-speed-review.md]
scenario_sources: [docs/design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md]
doc_role: plan
doc_topics: [design, preparation, numerical-execution, diagnostics, performance]
---

# Plan46: Execution speed and numerical efficiency

## Purpose and baseline

This standalone series captures the [execution-speed audit](../design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md): AF01–AF19, the reconciled external F01–F12 and U01–U26 recommendations, RC01–RC11 and scenarios S01–S09. The [external review](../pse-arrow-execution-speed-review.md) supplies additional numerical rationale and comparison families where the audit retains them. The audit's corrections govern this series; an external illustration is not automatically a requirement or an implemented capability.

The functional target remains the Process Simulator profile's *Functional target* and blueprint §0/§0.2: interactive value edit/re-solve without structural rebuild, studies/sensitivities/optimization/fitting, recycles, dynamics from the same definitions, extension and actual-dependency-local development. Core 3.4, efficiency heuristics 1.0 and Process Simulator 1.5 guide the design qualitatively.

**Interface-checked / source-inspected:** authoring baseline `5a009dccc7023e0142784892665c58bd20fbc305`, also the reviewed production baseline. The two supplied reviews and audit evidence were untracked before authoring and remain unchanged. Focused foundation inspection confirmed enclosing provider-cache hits, post-call IDAS checkpoint enforcement, canonical-wave resource sizing, per-sample append queue creation and whole-reference work inside the actual source-delta path. These are source observations, not new runtime qualifications. The audit's short native control failed during compilation; no numerical or speed measurement is inherited from it.

**Authorized execution, 2026-10-18:** the maintainer approved the detailed execution plan for the remaining C–G scope. RC04–RC11 targets and their bounded investigation/adoption routes are selected alongside completed RC01–RC03. Unknown numerical/platform winners remain investigation outcomes, not assumed implementations. Other plans retain their own scope/status; completed plans are not additional backlog and this series does not supersede them.

## Combined target and ownership

Keep authored physical meaning, mathematical preparation, provider invocation, native numerical algorithms, scientific workflow composition and publication at their existing owners. Remove unnecessary work at those owners instead of introducing another cache, AD system, scheduler or native-process framework.

The shared distinctions are structural preparation versus current values; immutable descriptor versus actual attempt provider; reusable numerical data versus required current effects; symbolic plan versus fresh matrix-specific factor; neutral allocation versus accepted numerical start/history; provisional dependency commit versus accepted stage; and provisional sample versus authentic durable prefix. Data lifetimes do not prescribe one new object per distinction.

Retain three distinct objectives: interactive edit-to-answer latency, one large-case latency and fixed-resource study/fitting throughput. Each inquiry names which objective it evaluates. Throughput comparisons hold total admitted resources and requested output/scientific demand comparable; a faster isolated case does not establish better throughput.

| Companion | Responsibility | Boundary supplied to other work |
| --- | --- | --- |
| [46a](46a-provider-effects-and-implicit-products.md) | Effect-safe providers and selected implicit products | Actual invocation validity, demanded actions, admitted fill storage and exact factor/product association |
| [46b](46b-mathematical-products-and-callback-reuse.md) | Mathematical layouts, products and callbacks | Compact/contracted execution, successful preparation reuse and atomic completed callback products |
| [46c](46c-root-transformations-preconditioning-and-responses.md) | Root transformations, preconditioners and response factors/rank | Consumer-qualified reconstruction, library-owned linear choices and truthful independent response products |
| [46d](46d-completion-scheduling-and-fitting-balance.md) | Block scheduling and fitting balance | Prepared dependency readiness, actual-overlap admission, reserved aggregate work and qualified private ownership |
| [46e](46e-dynamic-actions-and-checkpoint-lifetimes.md) | Dynamic actions and checkpoints | Actual DAE operator, effect-safe action/matrix reuse and enforced native checkpoint population |
| [46f](46f-source-publication-and-result-access.md) | Source deltas, live output and result access | Dependency-valid incremental source products, compact samples, actual ACK prefixes and independent decoder lanes |
| [46g](46g-investigations-qualification-and-design-publication.md) | Shared observations, platform comparisons and assembled closure | Decision-relevant short controls, bounded conditional adoption and final qualification/publication |

This coordinator owns finding dispositions, investigation decisions and operator rule decisions. Companion Work packages tables own package progress. Links to several packages describe contributors, not competing finding owners. Enduring contracts stay in their architecture/ADR owners; G07 schedules their proper publication after implementation.

### Foundation assessment and preservation

The existing math/native/workflow/publication boundaries can support the target. Necessary foundation corrections are effect enforcement through enclosing caches, explicit provider action composition/admission, factor/current-product associations, actual live-set admission and source binding-dependency reuse. A declaration alone is not a working prerequisite: the supplying consumer integration and its targeted test must exist before dependent execution.

Preserve:

- Complete physical inputs, model units/bases/reference states, original rows/bounds/balances and fresh scientific assessment. Ordinary engineering accuracy remains **0.001** where currently selected; work, integration, derivative/backward error, rank and KKT thresholds retain separate meanings.
- Original domain/smoothness obligations, branch selection and inactive-bound derivative neighborhoods. Symbolic cancellation or zero weights do not erase required checks; missing products are unavailable, never fabricated zero.
- Current provider effects even at identical numerical inputs. A retained completed product can survive a typed refusal without becoming authorized for the next invocation. Pure arithmetic reuse remains useful.
- Native ownership of iteration/globalization/factors, actual matrix/shift semantics, current clocks, finite aggregate work, bounded residency and actual cancellation/drain. No custom Newton, factorization or integrator is proposed.
- Existing study completion scheduling, continuation/start authority, serial initialization retention, fixed-layout KLU symbolic retention, selected dynamic axes and consumed physical-premise reuse. Their success does not qualify different owners automatically.
- Deterministic visible reductions/errors where promised, complete accounting of issued work, actual ACKs/fences, per-attempt/sample identities, frozen read permission and independent escaped lifetimes.

## Rule decisions

**RC01–RC11 accepted by the operator on 2026-10-18.** RC04–RC11 were selected by the maintainer’s instruction to implement the detailed remaining-scope execution plan; their bounded routes and exclusions remain in force. The [create-plan skill](../../.codex/skills/create-plan/SKILL.md#confirm-rule-changes-with-the-operator) requires: “Before fixing the target design, present each item to the operator with its consequence and obtain an explicit accept or reject.” These rows are concrete reviewable recommendations; authoring or approving the earlier preparation plan is not their acceptance. Acceptance selects these boundaries or bounded investigation/adoption routes, not production execution or an unknown solver/hardware winner. RC08 retains an existing requirement rather than changing it.

| Item | Recommended decision and consequence | Route before dependent implementation | Decision / date |
| --- | --- | --- | --- |
| <a id="rc01"></a>RC01 | Adopt explicit selected columns/JVP/VJP/weighted-second provider demands, composed through the real provider lift and admission. Keep full physical values, complete state solution and full jets when consumed; unsupported products refuse. | A03–A04/B02–B04; blueprint §§7.3/14.3, proposed ADR-0193; short ADR for new kernel/binding contract where required, architecture revision before changed consumers. | accepted / 2026-10-18 |
| <a id="rc02"></a>RC02 | Bind fresh attempt providers with an immutable descriptor lifetime and explicit fill/borrow completion. Separate invocation validity from numerical reuse; preserve uncontrolled boundary checks. AF19 is corrected under the current effect rule, not a relaxation. | A01–A02/A05; blueprint §14.3; amend relevant proposed rationale or short contract ADR as required. | accepted / 2026-10-18 |
| <a id="rc03"></a>RC03 | Adopt exact point/matrix/factor/first-product association, successful instantiation state and effect-safe assembled completion. Preserve callback atomic staging and final original assessment. | A06/B01/B05; blueprint §§7.5/14.3/18.2; proposed ADR-0193/ADR-0144; implementation-only savings need no invented decision. | accepted / 2026-10-18 |
| <a id="rc04"></a>RC04 | Qualify root transforms by complete primal reconstruction and original bounds/rows, separately from optimizer dual/KKT recovery. Implement qualified alias/affine reductions; contraction/condensation remain bounded comparisons. | C01; blueprint §§7.5/15/18.10, proposed ADR-0145; transformation pass decision before adoption, preserving accepted presolve rationale unless explicitly superseded. | accepted / 2026-10-18 |
| <a id="rc05"></a>RC05 | Adopt dependency-local successful block commit with actual-overlap admission, pre-dispatch aggregate work reservation, deterministic visible outcomes and atomic stage acceptance. Retain only backend-qualified neutral scratch. | D01–D04; blueprint §§17.1/17.6/18.8, proposed ADR-0166; update transactional initialization rationale through its required decision route before changing wave semantics. | accepted / 2026-10-18 |
| <a id="rc06"></a>RC06 | Permit a separately named sparse regularity/rank product only when its consumer accepts its method/scaling/cutoff. Keep existing dense SVD meaning and independent sparse actions. No QR/SVD equivalence assumed. | C05/I05; blueprint §15.4/§19.4; short new-product/binding decision if qualified, Python/metadata decision route if those contracts change. | accepted / 2026-10-18 |
| <a id="rc07"></a>RC07 | Investigate/adopt qualified threaded unsymmetric factors or existing reconstructible native-child execution only where complete-route benefit justifies actual team/ABI/storage/callback/drain costs. No new process framework or universal default. | C03/G02/I15; blueprint §18.8 and serial adapter bindings; new native binding decision before adoption, broader MPI/distributed scope excluded. | accepted / 2026-10-18 |
| <a id="rc08"></a>RC08 | Retain and enforce the configured checkpoint population before the native operation can exceed it. Preserve native integration/interpolation and accepted-step/replay ownership. Different checkpoint policy remains conditional. | E01; blueprint §13.6, existing adjoint rationale. Enforcement repair requires no rule weakening or new numerical algorithm. | accepted / 2026-10-18 |
| <a id="rc09"></a>RC09 | Permit bounded physical live-sample batching after selecting explicit visibility/flush boundaries. Preserve public original sample ordinals, provisional/selected attempts, authentic contiguous ACK and existing explicit detachment. No loss, automatic detach or early ACK. | F02–F03/I11; blueprint §§13.6/19.2; publication/visibility decision before relaxing per-sample flush; public boundary changes take their required route. | accepted / 2026-10-18 |
| <a id="rc10"></a>RC10 | Retain ordinary deployment-local response execution and conservative strict opaque-fill refusal. Investigate bounded producer knowledge/alternative factor ownership for actual strict consumers; do not weaken policy or claim completeness from RSS. A policy relaxation requires a new concrete decision. | C06/I06; blueprint §18.8, WorkAdmission storage contract; a producer-qualified improvement may proceed within this row, policy changes may not. | accepted / 2026-10-18 |
| <a id="rc11"></a>RC11 | Permit bounded realization investigations, preserving generic arithmetic/ISA by default. Hardware/precision/JIT adoption needs explicit support/equivalence and stop/drain contracts; relaxed precision, FMA/ISA change or GPU production support requires a concrete selected-profile decision. PGO alone does not change science. | G03–G05/I16–I18; blueprint §14.3, CPU policy and proposed ADR-0193/ADR-0144; appropriate decision/review before changed arithmetic/platform promises. | accepted / 2026-10-18 |

Operator rejection retains the current rule and requires adjusting only dependent scope, not silently abandoning the scientific target. Unknown winner or unmet eligibility is not rejection, acceptance or implementation. No blanket authority to relax accuracy, resource guards, provider effects or durability is sought.

Current rationale routes include proposed ADR-0180/ADR-0186 for output/ACK, ADR-0185 for native storage/reuse, ADR-0166 for concurrency and ADR-0144 for selected mathematics/response. Amend a fitting proposed record before implementation; if a selected change contradicts an accepted decision, use supersession instead. Accepted ADR-0110 dynamics, ADR-0118 SVD/statistical meaning and ADR-0083 class-specific execution are preserved by default; new in-bound bindings may take the short-ADR route without contradicting them. Formal ADR status is separate from implementation evidence. The authoring task did not edit ADRs. Authorized execution subsequently amended the fitting proposed records before dependent source changes; accepted records remain immutable.

## Execution dependencies and useful route

This is prerequisite ordering, not a requirement to execute whole documents alphabetically.

1. **Early correctness/foundations:** A01 fixes effect bypass; E01 enforces checkpoint population. G01 supplies the minimal selected observation/test setup needed by each package, without a new diagnostics platform or environment proof campaign. Independent source/readiness/scratch work may proceed when its own prerequisites are available.
2. **Demand and numerical reuse:** A02–A03 establish actual binding/product meaning; A04–A06 and B01–B04 integrate selective/compact/contracted execution. B05 consumes effect-safe completion before wider callback reuse. C04 response symbolic/scratch improvements do not depend on choosing a new rank product.
3. **Scheduling and native improvements:** C01/C02 can progress independently. C03 compares already available alternatives before new bindings. D01/D02 establish actual dependency and overlap/work semantics before D03/D04 dispatch/retention. D05 evaluates fitting affinity separately. E02 consumes the product capability it actually uses, then E03/E04 compare DAE/checkpoint policies.
4. **Source/publication:** F01 incremental references require complete binding dependencies. F02 encoding relocation can preserve per-sample flush; F03 consumes its buffers and accepted visibility boundary. F04 independent decoder lanes and F05 cache granularity are separate comparisons.
5. **Conditional realization and closure:** G02–G05 execute only their bounded inquiries, never postpone concrete correctness fixes. G06 runs the assembled selected qualification once functional scope/adopted routes are implemented. G07 publishes enduring design and disposition before any retirement.

Shared edit surfaces need coordination even when logical work is independent: A/B/E touch math demand/provider assembly; C/E touch factor storage/native settings; D touches shared task admission/session owners; F/G touch publication and observation. One coordinator owns manifests/generated schema declarations and cross-owner contracts. No compatibility production path is retained after a proven replacement; tests/callers/fixtures of deleted mechanisms are removed with it. A supported alternative solver/product remains only where it has its own consumed meaning, not as legacy evidence.

## Finding dispositions

Rows remain open until their corrective contracts are implemented and exercised. Package scheduling and rule acceptance do not establish implementation. This is the single current disposition owner; AF identifiers link to the audit, while packet identifiers name different objects.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| AF01 | S01/S02 | resolved | 46a/A03–A04 | Tested: actual selected/action demand reaches compiler, provider/lift/admission and generic composition; dense128 narrow products avoid unused complete jets. 46a Outcome records limits and acceptance. |
| AF02 | S02 | resolved | 46b/B02–B03 | Tested: compact first results and frames, interpreted/native, branches/zeros/physical guards and selected symbolic/Taylor comparison; companion checkpoint owns scoped counts and acceptance. |
| AF03 | S02 | resolved | 46b/B04 | Tested: retained first derivatives with supported pairs, and directional weighted-gradient construction for complete pair support; dense/sparse/shared controls retain original obligations. B04 owns bounded I02 comparison and its limits. |
| AF04 | S03 | resolved | 46c/C01 | Tested: primal-only affine/alias reconstruction and full original assessment in actual root/conditional consumers; exact-zero, tiny-coefficient, bounds and KKT separation controls. 46c/46g own scoped acceptance. |
| AF05 | S04 | resolved | 46c/C02–C03 | Tested/Measured: allocation-free warm BlockFactor application, fresh numeric validity, KINSOL/FERAL/PETSc short comparisons and derived NASM controls. No universal default winner; 46c owns conditions. |
| AF06 | S05 | deferred | 46e/E02–E03 | Implemented/Tested: dynamic JVP/VJP, retained CSC refill and checkpoint/action consumers. Stronger coupled DAE preconditioning remains unqualified. Owner 46e/E03; reopen on representative linear/fill dominance with current-shift algebraic regularity and a library-owned finite-storage binding. |
| AF07 | S06 | resolved | 46c/C04–C05 | Tested: exact immutable symbolic scope, private numeric/scratch per point, both owner-drop orders, current failure invalidation and strict preflight. Full scaled SVD meaning retained; 46c owns action/rank controls. |
| AF08 | S01/S02 | resolved | 46a/A02/A05 | Tested: immutable binding and atomic fills remove repeated descriptor work while preserving current effects, failure/reset and caller-owned storage. 46a owns acceptance. |
| AF09 | S01/S02 | resolved | 46b/B01/B05 | Tested: successful-order instantiation and exact completed assembly/matrix reuse remove redundant work; provider effects and foreign-output atomicity remain. 46a verification/46b Outcome own acceptance. |
| AF10 | S02/S09 | resolved | 46b/B03/B06; 46g/G03 | Implemented/Tested: compact support-shaped frames, compiler-established movement and scalar lookup improvements (46b/B03/B06). I16 completes source-led no-adoption; changed pure batch/constructor/ISA premise is required for SIMD/JIT. |
| AF11 | S07 | resolved | 46d/D01–D04 | Tested: completion-driven solve/initialization, live resource envelope, whole-child phase permission, conservative unknown-work reconciliation, canonical draining and atomic original stage assessment; 46d owns joined controls. |
| AF12 | S08 | deferred | 46f/F02–F04 | Implemented/Tested: off-callback selected capture/encoding, leased queue lifetime and cancellation-aware decode admission. Per-sample flush/ACK and serial protected decoding remain. Owner 46f/F03/F04; batching needs a finite explicit visibility contract, parallel decode needs complete protected-query benefit plus independent charged off-poll owners. |
| AF13 | S01 | resolved | 46a/A05 | Tested: retained residual/projection/verification scratch, borrowed internal completion and original contribution-wise solve checks; failure/reset/current-effect controls pass. |
| AF14 | S01 | resolved | 46a/A06 | Tested: exact matrix/factor/first associations reuse compatible upgrades and invalidate for affine use, changed child/configuration and failed verification. |
| AF15 | S05 | resolved | 46e/E01 | Tested: IDAS initial/subsequent checkpoint limits and Diffsol pre-clone enforcement, latched refusal, reset/current/peak and unchanged interpolation/physical output. 46e owns exact native controls. |
| AF16 | S06 | deferred | 46c/C06 | Tested: current strict consumers refuse before derivative effects; Local responses use the optimized scope. Owner 46c/C06; reopen only for a real strict consumer and complete producer-qualified opaque-fill/storage bound or qualified alternative factor owner. No policy relaxation. |
| AF17 | S01/S09 | resolved | 46f/F01 | Tested: actual source-delta reference/canonical reuse tracks successful and absent/shadow/import/binder/type/physical premises. Full metadata-map copy and ordered identity framing remain explicit receiving work; 46f/F01 owns controls. |
| AF18 | S07 | deferred | 46d/D05 | Tested/Measured: compact lane inventories and controlled manual affinity retain scientific products/order and remove observed idle skew. Production stride remains: no qualified general source/work cost. Owner 46d/D05; reopen for a complete fit-local cost/ownership model and measured construction/residency/throughput benefit without replication. |
| AF19 | S01 | resolved | 46a/A01 | Tested: real enclosing selected-partial/weighted/action requests rerun operational selector effects and refusals at unchanged keys, while pure hits remain eligible. |

The audit's [finding definitions](../design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md#4-findings-and-corrective-contracts) and [scenarios](../design_review/reviews/design_review_execution-speed-review-audit_2026-10-18.md#5-revealing-change-and-workload-scenarios) remain the source meanings. A conditional inquiry can finish with no adoption; an associated structural defect remains open/deferred with an accountable owner and observable trigger, never falsely resolved. G07 cannot claim every finding corrected if it accepts a retained limitation.

## Investigation outcomes

Required bounded inquiries are listed here; their package owns execution progress. Record the selected decision and its consequence here after the inquiry. A new broad capability or changed rule beyond RC scope returns for a concrete decision.

| Inquiry | Decision question | State | Decision / work owner | Completion or next trigger |
| --- | --- | --- | --- | --- |
| I01 | Which compact first-frame realization fits support, branches and providers? | resolved | 46b/B03 | Support-shaped frames and qualified sparse original-input First-region symbolic partials adopted; dense/dependent/control/provider regions retain Taylor. Interpreted/native nonlinear and failure controls passed; no coloring service. |
| I02 | When do contractions or retained partials beat flat pairwise HVP construction? | resolved | 46b/B04 | Adopt one-axis directional weighted-gradient construction for complete pair support; retain supported pairs with reused first derivatives for sparse/mixed support. Short dense/sparse/shared controls establish equivalence and separate preparation/operations/storage; no universal timing threshold or speedup claimed. |
| I03 | Which primal transforms/formulations remove full original work safely? | resolved | 46c/C01 | Primal-only alias/affine execution qualified in native roots and joined conditional consumers; exact-zero filtering and full original recovery. No unqualified nonlinear condensation/coalescing. |
| I04 | Which existing or stronger preconditioner/factor fits coupling/fill? | resolved | 46c/C03 | KINSOL KLU/BlockGMRES, FERAL GP/Markowitz and PETSc ILU/NASM controls passed with no global default winner; larger/threaded binding requires actual dominance and qualified ownership. |
| I05 | Does a real consumer need distinct sparse regularity/rank semantics? | resolved / retain SVD | 46c/C05 | Existing public spectrum/covariance consumes full scaled SVD; successful LU is not cutoff rank. No current consumer or qualified SPQR binding warrants another rank API. |
| I06 | Is strict sparse-response eligibility needed and supportable by a producer? | resolved / retained limitation | 46c/C06 | Strict consumers refuse before derivative effects; Local response remains optimized. Opaque fill lacks a complete producer bound; no policy relaxation. |
| I07 | Does cost-aware experiment affinity outperform static stride without replication? | resolved / retained limitation | 46d/D05 | Skewed four-experiment/three-trial controls preserve science and expose affinity benefit, but no qualified production cost model; retain stride and compact lane ownership. AF18 owns the trigger. |
| I08 | Which dynamic consumers benefit from direct actions versus a completed matrix? | resolved | 46e/E02 | Actual JVP/VJP, effect-safe completed-matrix reuse and selected dynamic consumers qualified by native and joined .001-accuracy controls. |
| I09 | Which DAE preconditioner fits actual algebraic coupling and shift? | resolved / retained limitation | 46e/E03 | Coupled scheduled index-1 KLU/None/Jacobi controls passed. No qualified coupled Krylov block/Schur binding; reopen on representative linear/fill dominance with current-shift ownership. |
| I10 | Which supported checkpoint/forward-adjoint policy trades memory/replay well? | resolved / current policy | 46e/E04 | Short scheduled sampled functional favors no new scheduler/default; retain explicit forward/adjoint demand and spacing250. Larger objective/horizon inquiry needs actual replay/memory bottleneck. |
| I11 | What live batching boundary improves work without violating visibility/ACK semantics? | resolved / current visibility | 46f/F03 | Retain authentic per-sample append/flush/ACK. Optional empty queue-control reuse adopted; cross-sample batching requires finite consumer visibility boundaries. AF12 owns the trigger. |
| I12 | Do independent file-affine decoder lanes/prefetch improve selected reads? | resolved / retained limitation | 46f/F04 | File-affine primitive reuse removes most observed opens; two lanes add charged state and do not uniformly help sparse reads. Retain production serial protected decoding pending complete-query ownership/benefit. AF12 owns the trigger. |
| I13 | Which source dependency closure permits narrow reference/canonical reuse? | resolved | 46f/F01 | Resolver-issued complete lexical/physical premises permit narrow reference/canonical reuse. Absence/shadow/import/deletion/physical controls pass; metadata-map copy/ordered identity framing remain. |
| I14 | Can protection/cache payload granularity retain useful work without undercharging? | resolved | 46f/F05 | Cache immutable exact source payload/receipts only; each caller obtains fresh independent current protection. Escaped lifetimes, cache eviction, overlap and uncertain-release controls pass; no budget increase. |
| I15 | Does inner threading or existing native-child isolation beat current outer scheduling? | resolved / current routes | 46g/G02 | Serial native restrictions and restricted reconstructible KINSOL child retained; reopen on demonstrated gate/factor dominance and qualified binding/team/drain. |
| I16 | Does eligible SIMD or a changed JIT/subprogram premise justify adoption? | resolved / no adoption | 46g/G03 | No changed JIT stop/ISA/Taylor premise; retain interpreted/conservative scalar C++. Actual pure homogeneous consumer and qualified constructor required to reopen. |
| I17 | Do actual release/dist/PGO artifact changes improve hot complete routes? | resolved / no adoption | 46g/G04 | Actual dev/release/dist/generated/native artifacts distinguished; no stable hot Rust target for PGO. Reopen on representative hot-artifact attribution. |
| I18 | Is there enough resident/batched work to justify a GPU target? | resolved / no adoption | 46g/G05 | Current adaptive callback consumers are host-owned; exported CUDA transfers/synchronizes each call. No supported resident batch demand; follow-up requires named dominant batch/precision/lifetime contract. |

## External recommendation coverage

External numbered findings map to audit findings and packages: F01→AF01/A03–A04; F02→AF02/B02–B03; F03→AF03/B04; F04→AF04/C01; F05→AF05/C02–C03/G02; F06→AF06/E02–E04 plus added AF15/E01; F07→AF07/C04–C06; F08→AF08/A02/A05; F09→AF09/B01/B05 plus AF14/A06 and AF19/A01; F10→AF10/B03/B06/G03; F11→AF11/D01–D04; F12→AF12/F02–F04. All original remedies are interpreted through the audit's corrections and preservation constraints.

| Audit coverage references | Owning treatment |
| --- | --- |
| U01–U03 | Shared distinctions/preservation; A03–A06, actual demand versus full physical values and dense implicit response. |
| U04–U05 | B03–B04/I01–I02; conditional coloring and contracted products, distinct exact-entry/limited-memory semantics. |
| U06–U08 | C01/C03 and D03; formulation, valid starts/continuation/recycles and existing native numerical controls. |
| U09 | C03/G02/I15; conditional threaded roots/processes under current owners. |
| U10–U11 | E02–E04/I08–I10 and C05/I05; sensitivities/checkpoints and separately named rank. |
| U12–U14 | D01–D05/G02; preserve study frontier, distinguish fit experiments/profile chains and existing compute lending. |
| U15–U18 | F01–F05/I11–I14; output/read/source/cache contracts at their actual owners. |
| U19–U22 | Exact library qualification in each packet; G02–G05/I15–I18; full eligibility and conditional deployment choices. |
| U23–U26 | Dependency route, G01/G06/G07, numerical preservation and explicit RC decisions. |

## Verification and completion

**Planned verification:** packets use affected compile checks and targeted unit controls with explicit Arrow force-validation where consumed. Replaced mechanisms/callers/tests/fixtures are deleted with the passing replacement. G06 selects relevant assembled native, workflow, Python, component, performance and manual checks once all adopted functional scope is integrated, including full continuing Clippy/hygiene at that point. G07 owns final architecture/status publication; no per-command ledger or documentation-only product reruns.

Short initial controls measure eliminated work and same-result operation, not a numerical speed SLA. Existing diagnostics supply actual components/RHS, factors, callback traffic, steps/rejections, waits, encoded/decoded/ACK work and complete coverage where available. Inclusive spans, reservations and RSS are not additive/exclusive equivalents. Inspect unexpectedly long runs; use cooperative stop and actual drain. Repeat only when variability can change the choice. Long-horizon/platform follow-up requires a named deciding premise.

**Completion:** every required packet has landed and its consumers/retirement/targeted controls are complete; required inquiries have a supported decision and any qualified in-bound adoption is integrated; RC outcomes and finding dispositions are explicit; required selected assembled checks meet the zero baseline. Report exact failures while keeping G06 and affected product acceptance incomplete until repaired. A deliberately unselected capability/retained limitation must have explicit disposition, ownership and trigger; it cannot turn a failing required check into success or be described as implemented. Scientific and architectural acceptance remain separate.

The original authoring task required only document publication checks and root `just turn-end`. The subsequently authorized execution includes the functional/static qualification above. No full wheel campaign, environment/provenance attestation or resource-default relaxation is revived; test-route repair remains bounded to selected G01 checks.

## Historical A/B checkpoint — 2026-10-18

**Implemented / targeted Tested, 2026-10-18:** all A01–A06 and B01–B06 packages and their
necessary enabling consumers are integrated. I01/I02 are resolved bounded implementation
choices. [46a verification](46a-provider-effects-and-implicit-products.md#verification-and-handoff)
records the exact selected compiler, compiled math/kernel/native callback, native root/runtime
and fitting commands and zero-baseline results; [46b Outcome](46b-mathematical-products-and-callback-reuse.md#outcome)
records construction choices and repaired integration failures. Independent review corrections
are integrated. Owned worker processes drained and all three owned worktrees were removed after
integration; unrelated work and shared registries/services were preserved.

**Interface-checked, zero baseline:** `BASH_ENV='' PSE_TEST_RESOURCE_STATE=/tmp/pse-plan46-root-resources-20261018 just check-solver-contracts`
passed with 0 errors and 0 warnings. It checks `pse-backend-native`, `pse-runtime` and
`pse-compiler`, all targets, `pse-runtime/native-solvers`, and explicit
`pse-runtime/force-validate` using continuing Cargo check. It does not execute those native
solver journeys. Earlier scoped constructor/fixture/import warnings and compile failures were
corrected; the final result is against zero rather than a retained baseline.

**Tested, documentation/tooling scope, zero baseline:** root `just turn-end` passed both
ADR-index and formatting. `just docs` published 502 chapters with scoped search;
`scripts/pse-env --docs -- python3 -m scripts.document_lifecycle validate` passed with
0 errors and 705 native scope items validated. Blueprint revision 209 and proposed ADR-0193
publish the selected implemented contract; formal ADR status is unchanged. Existing uncommitted
document authoring and review inputs remain in the working tree.

The operator accepted RC01–RC03 for this endpoint. AF01/AF02/AF03/AF08/AF09/AF13/AF14/AF19
are resolved; AF10 retains only its G03 contribution after completed B03/B06. Minimal selected
G01 support used existing force-validated/native-local routes and private registries, without
changing environment defaults. At that A/B endpoint RC04–RC11 and other companions remained unselected; the subsequent remaining-scope authorization below selects them. G06 assembled
qualification/static checks and G07 whole-series closeout remain open; no broader completion
or universal speedup is claimed. Completed A/B companions remain required by those live consumers.

## Current checkpoint

All 39 A–G packages and the I01–I18 bounded inquiry decisions are complete for their
adopted scope. Independent corrections are integrated on main. All three owned worktrees
were verified fully integrated and removed with their branches; no worker is still editing.

G06 selected native, mathematical, conditional/recycle, joined canonical and public Python
acceptance is complete. Original failed runs and repaired affected selections remain distinct
in [46g Outcome](46g-investigations-qualification-and-design-publication.md#outcome).
Scope-end hygiene repairs, full continuing default/no-default/native Clippy, final Ruff/Pyrefly,
governance/generation, native consumer lints and 419 depth-two plus 33 no-default compile
checks meet the zero baseline. This is selected affected-scope acceptance, not a new
whole-product/parity/platform or statistical-performance qualification.

AF01–AF19 have explicit dispositions: 15 resolved; AF06, AF12, AF16 and AF18 retain the
production limitations and observable triggers in the disposition table. Every required
bounded inquiry has a decision. No unqualified numerical/platform winner, strict producer
bound or general fitting-affinity policy is invented to close a package.

G07 transfers enduring meaning to blueprint revision 211 and its actual contract/rationale
owners. The highest coordinator, consumed companions and reviews remain retained for their
live references and limitations; Plan46 is removed from current-work selection. Other active
plans and original review inputs remain unchanged. The owned test service is stopped and
drained, with data retained. Physical engineering accuracy remains 0.001 where selected;
per-sample ACK semantics and solver defaults remain. Environment/provenance proof is
permanently eliminated, and no full wheel campaign was restored.

## Outcome

Recorded 2026-10-18.

**Implemented:** completed the adopted A–G target at the existing provider, mathematical,
native, workflow, source and publication owners. The companion Outcomes describe what
replaced enclosing effect bypasses, complete unconsumed products, wave dispatch,
post-allocation checkpoint enforcement, callback encoding and optional-cache active owners.
No compatibility production path is retained for a replaced mechanism.

**Tested / Interface-checked:** [G06's selected acceptance](46g-investigations-qualification-and-design-publication.md#outcome)
records actual commands, conditions, initial failures, scoped repairs and exclusions. All
required selected obligations have passing acceptance against zero; the initial runtime and
Python selections are not relabeled clean complete reruns. Short comparisons establish their
observed work/result/residency tradeoffs rather than a universal solver or speed guarantee.

**Mistakes corrected:** missing selected starts and construction-time residency assumptions
were corrected without relaxing production admission. Cache lifecycle oracles now exercise
the implemented immutable-payload retention and independent read permission. The CLI test
setup explicitly initializes its default database; formatting-stable FFI status bindings
preserve safety documentation. Transpose controls retain both required provider visits.

**Deliberate deviations:** retained SVD rank meaning, strict opaque-fill refusal, serial
protected decoding/per-sample visibility, production fitting stride and current native/
checkpoint/CPU defaults where bounded inquiry did not qualify another route. The four
retained production gaps remain deferred with owners and triggers, not claimed implemented.
No changed engineering accuracy, long statistical campaign, new environment proof or full
release-wheel qualification was introduced.

**Implemented, publication:** enduring contracts and selected limits reside in blueprint
§§13.6/14.3/15/17/18.8/20.4/21.1/24.2 and the corresponding proposed ADR confirmations.
Formal ADR status remains unchanged and accepted records were not edited. G07 records
metadata/site publication and owned cleanup; unrelated scopes retain their own owners.
