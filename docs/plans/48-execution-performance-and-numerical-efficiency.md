---
title: "Plan48: Execution performance and numerical efficiency"
status: done
date: 2026-10-20
adrs: [ADR-0192, ADR-0193, ADR-0201, ADR-0202, ADR-0203, ADR-0204, ADR-0205, ADR-0206, ADR-0207, ADR-0208, ADR-0209, ADR-0210, ADR-0211, ADR-0212]
review_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md, docs/design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md]
scenario_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md]
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [design, preparation, numerical-execution, performance, diagnostics]
---

# Plan48: Execution performance and numerical efficiency

## Purpose and basis

Remove avoidable preparation, mathematical-product, native-lifecycle and coordination work
through the complete process-execution pipeline. Preserve scientific meaning, practical
acceptance and honest completion while improving cold first-result, warm edit/re-solve,
study throughput and demanded result availability. The
[independent audit](../design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md)
and its [recommendation register](../design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md)
are the basis: AF01–AF18, N01–N04, RC01–RC12 and all 162 register rows. Original external
R1/R2 proposals are interpreted through the audit's corrections, not adopted wholesale.

**Authoring baseline, 2026-10-20. Interface-checked:** current implementation at
`1ee81798a9b884f3054b685ee9629e2c8d124498`, affected source/architecture owners and retained
uncommitted review inputs. **Proposed:** all new behavior and packages. The audit's
retained-run interpretation is historical **Measured** evidence under its stated conditions,
not a measured benefit from these remedies. No new simulation, product implementation or
solver qualification was performed during plan creation.

The standard is Core 3.4, Efficient Architecture Heuristics 1.0, Process Simulator 1.5 and
the pse-arrow binding. The target includes value edits without unrelated structural rebuild,
studies/sensitivities/optimization/fitting, recycles/tears, dynamics from the same definitions,
unit/property/solver extension and responsibilities testable with their actual dependencies.
Architecture sections own current contracts; this series owns proposed work and, after
adoption, its dispositions. Other plans keep their scope/status. Document creation does
not authorize product execution, default changes or dependency publication.

## Reading route and ownership

| Companion | Responsibility and packages | Principal obligations |
| --- | --- | --- |
| [48a: Source checking and incremental preparation](48a-source-checking-and-incremental-preparation.md) | A01–A06; indexed checking, mutable overlays and immutable artifacts | AF01/AF02/AF18; I01–I03 |
| [48b: Registry and receiving validation](48b-registry-and-receiving-validation.md) | B01–B05; descriptions, domain/template reuse and receiving eligibility | AF03/AF04; I04–I06 |
| [48c: Mathematical products and compiled execution](48c-mathematical-products-and-compiled-execution.md) | C01–C08; contributor indexes, sparse products, complete validity and modules | AF05–AF08; I07–I11 |
| [48d: Native controls, starts and recycle policies](48d-native-controls-starts-and-recycle-policies.md) | D01–D07; native meanings, initialization and qualified method policies | AF12/AF13; I12–I20 |
| [48e: Dynamics, fitting and preconditioning](48e-dynamics-fitting-and-preconditioning.md) | E01–E08; private allocation, selected products and anchored windows | AF14/AF15/N01; I21–I27 |
| [48f: Scheduling, resources and result completion](48f-scheduling-resources-and-result-completion.md) | F01–F09; compatible dispatch, readiness, continuation/drain and effect lifetimes | AF09–AF11/AF17; I28–I33 |
| [48g: Diagnostics and review publication](48g-diagnostics-and-review-publication.md) | G01–G06; reliable current state, coherent observations and streaming output | AF16/N02–N04; I34/I35 |
| [48h: Qualification and design publication](48h-qualification-and-design-publication.md) | H01–H04; selected assembled acceptance and enduring publication | All selected routes; I36 |

Companions own package state. This coordinator alone owns finding/investigation state,
rule decisions and the next route. The coverage appendix is navigation, not another ledger.
Package identifiers are companion-local; 48a/A01 is not a review finding.

## Combined target and foundation assessment

Retain model/physical authority, library mathematics, native iteration, workflow composition
and result/diagnostic ownership. Improve physical work within those boundaries:

| Existing foundation / inspected constraint | Target consequence |
| --- | --- |
| Eager named-type environments; granular source/expression inputs and stripped Namespace already exist | Indexed demand-driven lookup with complete membership/absence dependencies and phase-local mutable overlays; extend incrementality rather than replace the checker |
| Repeated immutable descriptions/bootstrap/domain preparation; schema-pointer/templates/retained-contract paths already exist | Reuse immutable supplier products while keeping receiving registry/configuration/functions and invocation-sensitive eligibility current |
| Implicit/derived actions reconstruct columns/dense scratch; factor checks occur after reconstruction | Canonical sparse demand and complete current validity before elision; action vectors rebind without unbounded caching |
| Joint arithmetic and native compilation already exist with ordered barriers | Pack bounded co-used demanded functions without crossing guards/provider effects or preparing every order |
| Native lanes/teams, durable readiness and in-process renewal already exist | Correct compatibility/frontier/capture/access gaps before introducing duplicate pools, counters or supervisors |
| Dynamic storage, current trial and checkpoints have different lifetimes; fitting already has GN curvature | Complete neutral trial reset and private bounded windows; retain constraint curvature and demanded derivative eligibility |
| Lossy diagnostic updates can drop admitted phase closure | Reliable current lifecycle independent from optional history; coherent progress separate from exact local work/cancellation |

Checked source/package artifacts, receiving admission, mathematical structure, current
linearization, native trial allocation and acknowledged result effects have different
lifetimes. Each survives only its complete consumed premises. Existing owners/functions
can express these distinctions; six new services/types are not required.

### Preservation and boundaries

- Physical acceptance stays **0.001 where selected**. Native relative norms, stagnation,
  gaps, stationarity, integration errors and effort limits retain distinct meanings.
- Preserve dimensions/bases/reference states, envelopes, structural roles, normalization,
  source mappings, root/branch permissions, effect order and first relevant refusal.
  Value-valid is not First-valid. Preserve exponent-materialization protection until
  a replacement is qualified at the actual Symbolica release.
- Preserve canonical fingerprint framing and exact equality promises. Hashing hashes is
  an identity change. Arithmetic regrouping needs declared numerical/guard/diagnostic
  equivalence; bit identity only where actually promised.
- Reuse established immutable validity, not old receiving permissions. Fresh/unsafe/native
  construction can create internal validation obligations. Correctness closures explicitly
  activate force-validation.
- Mutable workers, seed bindings, native destruction and charges remain with actual owners.
  Last chunk is not terminal success. Retain post-success stop veto, panic reconciliation,
  producer drain, native exclusion, leases/fences, ACK truth and uncertain-outcome recovery.
- Environment/provenance proof remains permanently eliminated. Actual scientific/ABI/layout
  cache premises do not authorize host attestation, a proof service or broad installation work.

The audit's S01–S06 remain the scenarios. Source/import/data edits invalidate consumed
meaning and negative lookups; larger sparse workloads follow contributors rather than
parent scans; provider/order/branch changes rebind affected products; adapter changes absorb
native mechanics; skewed cancellation/publication preserves other progress and owned drain;
unit/workflow extensions reuse semantic/composition owners without duplicating policies.

## Rule decisions

**RC01–RC12 were explicitly accepted by the operator on 2026-10-20**, selecting
“Accept all 12 recommendations as written.” These are accepted target boundaries and
bounded inquiry routes. Acceptance does not choose unknown numerical/platform winners,
authorize product implementation, or permit broader changes discovered later. The retained-rule
alternatives explain the scope of each choice, not a pending decision. Dependent adoption
still requires the row's concrete decision/design work and qualified inquiry outcome.

| Rule | Recommendation and consequence | Decision / date | Route before dependent change | If rejected |
| --- | --- | --- | --- | --- |
| RC01 | Correct KINSOL relative scaled-step/stagnation meaning; qualify native working controls with unchanged original residual/physical acceptance | accepted, 2026-10-20 | Blueprint §16.6 design correction; D01 specifies selected policy before implementation | Correct the false equivalence; retain conservative native settings without an absolute-step claim |
| RC02 | Permit immutable checked-artifact acceleration with complete dependencies and fresh receiving bindings; compare smaller artifacts with whole Salsa persistence | accepted, 2026-10-20 | Blueprint §14.4 design change; ADR for a selected new pass/storage binding where required; A05 | Lazy/granular in-process correction only; fresh processes still check packages |
| RC03 | Permit explicit coherent coalesced ordinary progress cadence/final flush and reliable admitted phase retirement, separate from optional history loss | accepted, 2026-10-20 | Observation contract/design decision at blueprint §23.1/§24.1 before G02; G01 reliable closure remains independently required | Keep per-return freshness; optimize local formatting/updates and reliable closure |
| RC04 | Permit pre-reconstruction product/factor reuse only with complete current linearization validity | accepted, 2026-10-20 | Mathematical kernel/assurance contract decision and blueprint §14.3 before C03 | Sparse construction remains useful; reconstruct coefficients before factor reuse |
| RC05 | Qualify an actual coupled index-1 DAE approximate preconditioner of the exact shifted operator, with refresh/singular-block behavior | accepted, 2026-10-20 | Short ADR for selected native/kernel binding and capability design before E05 adoption | Keep direct KLU and None/Jacobi; still implement selected Jacobi demand |
| RC06 | Investigate separately qualified vector/JIT/device realizations and compiled-default changes; adopt only a demonstrated eligible route | accepted, 2026-10-20 | Concrete realization decision, applicable ADR-0191/0193 route and blueprint §14.5 before adoption; C07/C08 | Scalar execution and bounded demanded-module packing remain |
| RC07 | Investigate restricted Ipopt same-structure reuse, including fixed/free/equality maps, value-dependent scaling, effective options and reset | accepted, 2026-10-20 | Native adapter decision/design at blueprint §18.3 before D04 adoption | Keep `warm_start_same_structure=no`; retain truthful compatible primal/dual seeds |
| RC08 | Investigate a qualified explicit recycle policy/default; do not preselect universal history/delay | accepted, 2026-10-20 | Policy/design decision at blueprint §17.4 after D06 comparison | Current defaults remain; indexed exchange and selectable acceleration still apply |
| RC09 | Investigate process-isolated or broader concurrent native capability for a concrete workload | accepted, 2026-10-20 | Native deployment ADR/design route before F08 adoption | Preserve MPIUNI/scheduler/TPI gates and actual native owner drain |
| RC10 | Investigate additional adapter-specific neutral reset or worker placement with complete private state and owner-thread lifecycle | accepted, 2026-10-20 | Selected reset/threading contract decision at blueprint §18.8 before affected D04/E01/F08 adoption | Reuse only qualified lanes/scopes; optimize immutable preparation/local scratch |
| RC11 | Investigate generated typed authored-document loading only if template/domain reuse is insufficient, retaining identical registry/SQL/receiving obligations | accepted, 2026-10-20 | Selected loader/boundary decision and blueprint §4 design route before B05 adoption | Optimize existing ingress/templates/retained contracts without a second validator |
| RC12 | Investigate compute/commit lifetime separation only for observed blocking, with bounded pending products and unchanged ACK truth | accepted, 2026-10-20 | Workflow/publication lifecycle ADR/design at blueprint §§19/20/24.1 before F06; any commit-contract change requires a separate concrete decision | Joined publication remains; improve dispatch/capture/actual crossings first |

These rows do not change identity framing, allow guard-after-fusion, transfer receiving
eligibility, relax physical acceptance or remove native safety gates by analogy. Newly
exposed impacts return as concrete decisions. Accepted records stay immutable; any required
supersession follows its decision route rather than an edit to history.

## Execution sequence and prerequisites

1. Apply the recorded rule decisions and schedule required ADR/design work before dependent
   implementation. Establish truthful native controls/receipts (D01–D03) and reliable phase
   retirement (G01) early; comparisons depend on those meanings.
2. A01–A04, B01–B03, C01/C02/C04/C05, F01–F03/F09 and G03/G05 supply bounded corrections
   independently. They consume actual receiving, support, compatibility and terminal-producer
   behavior, not merely agreed interfaces.
3. Resolve artifact, arithmetic, native-policy/reset and scheduling inquiries before their
   conditional adoptions. Local work removal does not wait for a persistent compiler,
   universal pool or platform expansion. C01 supplies indexes before D05 physical grouping.
4. E03 consumes actual private workers and shared capacity/affinity/joined drain; fresh
   sessions are valid. It consumes E01 retained reuse/E02 optimizations only when available,
   not as prerequisites to independence. E04 is independently useful.
   F04 shares only immutable admission; F05/F06 preserve durable
   identities. G01 precedes increased span volume; G02 requires the selected cadence.
5. Migrate actual consumers and remove proven replacements with targeted acceptance.
   H01 reconciles completeness, H02 runs revealing short cases, H03 qualifies selected
   assembled scope/final checks once, and H04 publishes enduring meaning and reconciles
   existing plans after implementation/qualification is complete.

Document order is not a synchronization barrier. Shared file ownership is separate from
logical dependency: assign one writer for generated declarations, math products, native
adapters, observation state and lifecycle metadata. No compatibility API or second
production path remains after a replacement is proven and callers have moved.

## Finding dispositions

The table distinguishes current implementation and focused acceptance from remaining
package work and assembled qualification; scheduling alone closes no finding.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| AF01 | S01/S02 | resolved | 48a/A01–A02 | Independent lookup/overlay outcomes and reduced examined-reference work; selected assembled acceptance passes in48h |
| AF02 | S01 | resolved | 48a/A03/A05–A06 | Narrowed dependencies/bounded retention; complete artifact design and qualified fresh-check/serial decisions recorded in I01–I03; selected assembled acceptance passes in48h |
| AF03 | S01/S02 | resolved | 48b/B01/B04 | Same identities/custom fallback; fewer repeated descriptions/bootstrap forces |
| AF04 | S01/S03 | resolved | 48b/B02–B03/B05 | Domain/template reuse, receiving eligibility and malformed-input rejection |
| AF05 | S02/S04 | resolved | 48c/C01; 48d/D05 | Contributor/request and boundary row/support consumers are indexed; CA01 growth and full boundary obligations pass in the 27-test repair selection;48c/C01 and48h own actual acceptance |
| AF06 | S02/S03 | resolved | 48c/C02–C03 | Sparse products and complete validity before reconstruction elision |
| AF07 | S02/S03 | resolved | 48c/C04 | Sparse flat direct HVP eligibility without losing entry/reverse routes |
| AF08 | S01/S02 | resolved | 48c/C05/C07 | Bounded named modules, fewer children and safe code lifetime |
| AF09 | S02/S04 | resolved | 48f/F01 | Compatible/incompatible dispatch and unchanged assessment |
| AF10 | S02/S05 | resolved | 48f/F02 | Transition-driven ready visits, continuation/retry order preserved |
| AF11 | S02/S05 | resolved | 48f/F03 | Sibling progress during skewed capture; continuation/producer drain retained |
| AF12 | S04 | resolved | 48d/D01–D02 | Separate KINSOL/cone obligations; original acceptance unchanged |
| AF13 | S04 | resolved | 48d/D03–D04 | Submitted primal, optional payload and internal reuse reported distinctly; incomplete Ipopt same-structure reset remains explicitly ineligible |
| AF14 | S03/S04 | resolved | 48e/E01–E02 | Current fixed-profile reset/failure discard, private scratch, distinct active coverage and exact original pool balance after job drain pass; unsupported backward/Second storage stays fresh |
| AF15 | S02/S03 | resolved | 48e/E03 | Actual private fixed-trial windows, ordered results and distinct active coverage pass their selected controls; selected assembled acceptance passes in48h |
| AF16 | S02/S05 | resolved | 48g/G02–G03 | Coherent current/final progress and reduced shared work |
| AF17 | S02 | resolved | 48f/F09 | Indexed transport preserves subsets/reordering/unknown-ID errors |
| AF18 | S01/S02 | resolved | 48a/A04 | Typed masking preserves field roles/signed zero/conservative fallback; the normal full generator publication now passes; selected assembled acceptance passes in48h |
| N01 | S02/S03 | resolved | 48e/E04 | Actual authored Jacobi selected setups equal all10 native setups, with positive selected-entry work and zero full setups; unavailable generated-rate uncertainty remains an explicit refusal |
| N02 | S05 | resolved | 48g/G01 | Reliable admitted phase retirement under optional-history pressure; 60 selected observation controls, zero failures |
| N03 | S02/S05 | resolved | 48g/G03 | Prepared progress IDs preserve all actual consumers |
| N04 | S02/S05 | resolved | 48g/G05 | Equivalent coherent publication with bounded row retention; nine focused publication controls, zero failures |

## Investigation outcomes

Companions specify finite question/options/evidence/stopping conditions. Completion means
qualified adoption or justified no-adoption/support limit with an observable reopening
trigger. Declining an alternative does not resolve an uncorrected finding. No unknown
method, format, allocator, platform or default is selected by this table.

| Inquiry | Decision question | State | Decision / work owner | Completion or next trigger |
| --- | --- | --- | --- | --- |
| I01 | Which checked artifacts and imported body/data dependencies support fresh-process reuse? | resolved: fresh checking | 48a/A05 | Complete payload/receiving design recorded; FunctionRecord roundtrip does not cover changed defaults/data/physical/absence. Retain fresh process checking; reopen for material restart cost after A01–A04 or a new consumed source/data/physical kind. |
| I02 | Does whole Salsa persistence improve on smaller immutable artifacts? | resolved: no adoption | 48a/A05 | Exact Salsa0.28.4 supports persistence, but this compiler has no complete wire or qualified owner restoration. Neither storage route adopted; reopen on a concrete restart workload or changed transitive restoration capability. |
| I03 | Which immutable checking phases can run in parallel? | resolved: serial owner retained | 48a/A06 | Actual immutable parser clone/drain/cancel probe passes; production compiler job admits one core. One cold 68-row dev probe: parse/stage636us, complete9836us. Reopen with actual multi-core source-read grant and a demonstrated parse-dominant workload; canonical error order retained. |
| I04 | Is a frozen catalog/archive useful after description/bootstrap fixes? | resolved: adopt complete generated products | 48b/B04 | ADR-0210 selects declaration-generated complete resolved products and a fresh live owner shell. The actual producer publishes the complete payload and both standard initializers now hydrate it. Five focused controls pass independent source equivalence, current receiving/fresh owners, custom empty registries and bad IPC refusal. The earlier partial-image gap and compiler-expensive fallible array construction were corrected in the producer. Selected assembled consumers remain with48h. |
| I05 | Which repeated validations truly share one receiving scope? | resolved: actual receiving owners | 48b/B03/B05 | Existing RunResult shares result_validation; engine operation caches already bind the actual scope. Preserve fresh contexts for distinct calls/activities/new execution_state; no broad equal-value/time scope fusion. |
| I06 | Can typed loading remove remaining work without a second validator? | resolved: exact-context generated typed admission | 48b/B05 | ADR-0202 scoped target uses generated typed projection only for exact supported declaration and rewrite-free current NativeValidation; custom/rewriting planners retain actual evaluation. Generation refuses unsupported facets. Focused guards/planner/load-path tests passed; exported Arrow remains independently checked. |
| I07 | What complete child validity permits reconstruction elision? | resolved: complete current First probe | 48c/C03 | Same immutable problem/layout, exact numerical key/point/state-First demand and all-residual/all-coordinate zero-First traversal must re-establish original validity and every complete seed-independent child trace before coefficient/factor elision. Missing/mismatched premises reconstruct; failed guards/cancel/reset/affine use invalidate. Zero/unit/subset/branch/changed-token/Value-only-guard probes and focused dependent implementation controls passed; selected assembled acceptance passes in48h. |
| I08 | Which arithmetic/instance/value-order reuse preserves ordered meaning? | resolved: current-pass pure sibling reuse | 48c/C06 | Reuse only last completed eligible pure sibling donor with exact gathered bits and sufficient product in the same current pass; no provider/effect/context donation or cross-pass cache. Ordered dependent regions remain. Opposite-sign/equal-value derivative, order, guard, cancellation and insufficiency controls pass in current math/compiled slice. |
| I09 | Which scalar artifact/cache/default improves cold/warm total behavior? | resolved: adopt disposable scalar cache; explicit compiled selection retained | 48c/C07 | ADR-0212 keys actual generated source, scalar compiler/FP/ABI profile, names and dimensions; bounded atomic images are copied into current private module owners and checked before use. Twelve selected scalar controls pass, including actual fresh-process hit/fallback, changed-input, corruption, stop and lifetime; the subsequent real concurrent-writer/eviction/receiving extension also passes1/1. Interpreted default and one-core sequential compile/link on misses stay unchanged; compiler fanout still needs distinct actual grants. |
| I10 | Which remaining provider contractions/reverse actions remove per-output work? | resolved: existing composed actions qualified | 48c/C08 | Actual128-input/two-output provider composition over two invocations uses2JVP+2VJP and, for Second,2weighted-second calls without full partial arrays. Distinct cotangent tangents still require distinct VJP under the current demand contract. Context/unknown-owner/branch/guard controls pass; no additional generic contraction demonstrated. Reopen an actual repeated cotangent/contracted product outside this qualified pair. |
| I11 | Does a concrete vector/JIT/device workload justify expansion? | resolved: scalar profile retained | 48c/C08 | Actual Symbolica3.0.1 scalar JIT/packed/interpreted polynomial probe includes common preparation, packing/output and16 warm passes for1/64/512 rows. JIT improves warm work here but public options select native host ISA rather than the portable scalar compiler target; no FP/ISA equivalence or production adoption inferred. No concrete device/distributed workload established. Reopen a coherent reuse workload plus explicitly admitted ISA/FP/cancellation/transfer owner. |
| I12 | Which native KINSOL working policy meets original acceptance? | resolved: native roundoff stagnation | 48d/D01 | Zero KINSetScaledStepTol selects native roundoff default; explicit positive values remain relative stagnation. Translated cubic near zero/large origins, shifted bounds, residual multipliers, cold/warm/accepted starts and explicit maximum-step refusal tested with independent original residuals. Physical0.001 unchanged; revisit native formula/scaling or demonstrated false stagnation. |
| I13 | Which conic working/refinement controls are sound per adapter? | resolved: one bounded Clarabel correction; POUNCE policy retained | 48d/D02 | ADR-0204 production controls pass: finite observed-native-residual/original-quality working correction, same original clock/remaining iterations, native failure/Limit kept distinct, SOC and PSD original checks, ordinary/refined late-publication cancellation and no stale prior candidate. Five new controls are within28/28 selected native passes; existing conic controls also pass. POUNCE's separately observed18-case SOC absolute/gated-relative behavior does not receive Clarabel's heuristic. No universal cone success or physical accuracy change. Reopen a concrete original-quality failure or newly exposed native original-budget control. |
| I14 | Which restart/barrier/options policy benefits supplied starts? | resolved: explicit existing restart profiles | 48d/D04 | Current Ipopt/POUNCE richer restarts and primal-only paths qualified in bounded native controls. On the six-coordinate perturbed simplex, good seeds took2 versus cold5/6 iterations, primal-only3, poor multipliers/barrier7–8. A stated0.001 barrier did not universally improve good starts. Preserve current explicit profiles and complete per-call reset; no global barrier change. Reopen a demonstrated poor-seed/limited-memory/native-option regression. |
| I15 | Can Ipopt retain structure including maps/scaling safely? | resolved: retain fresh spaces | 48d/D04 | Actual Ipopt3.14.20 same-structure initialization retains spaces/scaling and skips ordinary cache clearing. Current dimension/pattern signature does not bind complete value-dependent scaling/provider state. Compatible/incompatible classification, changed-gradient scaling and option-removal controls pass with warm_start_same_structure=no; retain it. Reopen complete native cache/scaling reset plus a demonstrated setup-dominant workload. |
| I16 | Which additional handles have complete neutral reset? | resolved: qualified fixed-profile IDAS storage | 48d/D04; 48e/E01 | Owner-affine Value/Forward reset uses fresh callback/execution validity and distinct finite coverage; failure/changed profile discards, backward/Second stays fresh. Native reset/current-cancel controls, actual nested fitting/profile populations and exact job-drain accounting pass. Other adapters retain their qualified existing reset/fresh policies; reopen a concrete complete reset contract and setup bottleneck. |
| I17 | When do grouped tiny/dense/sparse blocks and setup refresh help? | resolved: existing common Direct owner; explicit BTF owners | 48d/D05 | Sixty-four one-core separate/grouped populations and changed numeric generations pass original0.001, singular and recovery checks. Grouping removes setup work and increases resident peak. Existing default Direct already solves the common original case as one owner; BTF retains independently requested block permissions, overrides, starts and budgets. No timing threshold rejects grouping. Arbitrary subset coalescing still needs a complete shared permission/admission boundary. |
| I18 | Should the explicit recycle default/history/damping change? | resolved: explicit native controls retained | 48d/D06 | Six-coordinate contractions/alternating/slow/expansive domain-limited maps compare history0/1/3/5 and delay/damping with original residuals, complete map calls and storage. Larger history helps slow cases but not easy contractions; damping is case-dependent. Existing nonlinear/regime/failure controls retain actual physical meaning. Keep explicit per-request settings and current defaults, including history<=dimension; reopen a representative regime/quality regression. |
| I19 | Are library Wegstein/Broyden or derivative-capable Newton tears useful? | resolved: retain native map acceleration | 48d/D06 | Current CausalUnit exposes ordered values and independent original state, not a complete composition derivative or permission to change root selection. KINSOL owns qualified fixed-point/Anderson iteration. Pyomo6.10.1 Wegstein owns a different Python Network/Port/Var graph rather than this native map; existing PETSc/diffsol Newton routes require their admitted equation/operator contract. No new cross-model bridge or host iteration adopted. Reopen a concrete native whole-map library integration or complete composition derivative/root permission plus demonstrated benefit. |
| I20 | Which eligible sparse-direct/Krylov/native linear backend helps? | resolved: retain explicit qualified portfolio | 48d/D07 | Twenty-four one-core existing MUMPS/SPRAL/Pardiso/POUNCE-FERAL complete NLP solves across dense16/banded64/skewed32 and two numeric generations pass original equality/stationarity/inertia/backsolve checks at unchanged engineering policy. Cold/order and outer-method differences prevent an isolated linear winner; Ipopt internal factor RHS/solutions remain unobserved. Retain caller selection; reopen a representative material setup/factor bottleneck with actual operator and admitted team. |
| I21 | Which coupled shifted-DAE preconditioner is useful? | resolved: adopted BBD binding and public admission boundary qualified | 48e/E05 | Library BBD and historical coupled/action/runtime results retained; CA02 now admits once before public Initial effects. Public absent/insufficient/denied memory, current cancellation, direct reset and authored runtime BBD controls pass in the 27-test repair selection |
| I22 | Which IDAS corrector/h0/statistics/reset choices are safe? | resolved: explicit correctors/native h0 and coalesced statistics | 48e/E06 | Twelve ODE/DAE KLU/unpreconditioned/Jacobi simultaneous/staggered controls pass original analytic state/output and sensitivity-effect checks at local0.001, without a universal corrector winner. Actual batched statistics match native getters across initialization/advance/reset/failure. Native automatic h0 improves the stiff fixture's original output classification while taking41 versus38 accepted steps and61 versus65 residual evaluations; smooth work is identical. ADR-0207 selects the native automatic default with explicit Configured and positive IC orientation unchanged; IdasSettings3/Profile3 prevent reinterpretation of old omission meaning. The affected current-default/version/native lifecycle selection now passes26/26 with zero warnings or errors. Reopen a representative corrector/h0/setup bottleneck, not an isolated wall-time winner. |
| I23 | Can a narrower pure-ODE backward route reduce reset/IC work? | resolved: exact first-order ODE initial rates | 48e/E06 | ADR-0206 current guarded negative-transpose action supplies rates only when every actual state is differential and the backward problem is first order. ReInitB/quadratures/observation and segment jumps remain. Native-IC reference pairs retain49/138/139 steps while removing four nonlinear iterations in each observed case; gradients agree within about1.5e-8. Independent nonsymmetric/state/time/parameter action and failure/stop/DAE/Second eligibility controls pass, as do existing nonlinear adjoint/DAE/Second regressions. An original-invalid stiff forward trajectory remains invalid under both routes; no tolerance is changed or success promoted. Reopen a changed mass/formulation or current-action/adjoint regression. |
| I24 | Is LM/TR useful versus existing constrained Ipopt-GN? | resolved: qualified automatic residual LM adopted | 48e/E07 | Actual unboxed/unconstrained response and requested-contract eligibility, known workspace before effects, honest method/termination/work, persisted public metadata and fresh original assessment pass. Matched complete operations retain objective/gradient/covariance/Wald/predictions at unchanged practical accuracy; no universal speed or optimality certificate. |
| I25 | Which forward/adjoint policy fits demand and checkpoint budget? | resolved: retain demand-correct explicit selection | 48e/E07 | Actual scheduled nonlinear DAE sampled-functional controls compare forward/adjoint, selected/all axes and checkpoint3/25/250 at unchanged0.001 with direct/initial/schedule terms. LM/rank/Gram require actual response products; gradient-only limited memory can consume explicit adjoint and then fresh final responses. A parameter-count-only crossover lacks functional/output, checkpoint/replay and team premises. Reopen a complete demand- and budget-qualified selector, not a marginal timing win. |
| I26 | Are explicit inexact integration controls useful for fitting? | resolved: retain fixed original accuracy | 48e/E07 | Native local/endpoint estimates do not enclose the weighted objective and response-gradient error or provide acceptance forcing/upgrade evidence. A loose-tolerance trial loop would mix unqualified value/gradient meaning near flat/active optima. Adopted value-only demand/transactional upgrades remove work without changing accuracy. Reopen an actual objective+gradient-consistent error estimator; physical0.001 stays unchanged. |
| I27 | Which shooting starts/optimizer allocations can be reused? | resolved: explicit primal and fresh optimizer retained | 48e/E08 | Actual Diffsol/Ipopt tracking at standard engineering accuracy: ordinary/good/poor primal starts took6/3/7 iterations, all independently checked original objective near0.31417; API submission true and native-state reuse false. Changed bounds/options, native iteration-limit then fresh success, invalid/cancelled requests and equal-sized distinct causal anchor maps pass. Shooting has no complete compatible dual/restart-map or neutral optimizer owner; each call uses fresh Retained state and a current callback/execution. No dimension-based history transport or barrier default adopted. Reopen an explicit complete map/reset capability or reproducible setup-dominant repeated-shooting workload. |
| I28 | What immutable admission can local lanes share? | resolved: retain existing immutable sharing | 48f/F04 | Actual independent equal-basis lane preparations share the same SharedAllocation; changed source/provider/current-scope controls pass. Two views/observations require zero rebuilds while scopes remain distinct. No new HeldPreparationScope or PreparedLaneInputs sharing platform is justified. Reopen demonstrated remaining structural preparation cost with a complete immutable owner. |
| I29 | Which durable discovery/claim/wakeup crossings remain unnecessary? | resolved: retain current authoritative discovery/claim crossings | 48f/F05 | Actual canonical probe passes five idle and five independently acknowledged usable fixed occurrences after the resident-loan correction. Median observed action1.300806s, claim23.443721ms and inclusive publication978.369690ms; idle actions393–1330us. These scoped observations do not identify claim latency as the remaining dominant cost or justify a new push/subscription platform. Existing hints/repair/renewal remain; F06 separately addresses real joined-publication serialization. Reopen a distinct measured discovery/claim bottleneck or reconnect/readiness regression. |
| I30 | Does publication blocking justify a finite commit stage? | resolved: bounded Running/Publishing handoff adopted | 48f/F06 | Actual same-worker width-one held-publication refill, finite saturation, late-arrival discovery, late export failure, cancellation/no-payload and lost-ACK/prior-failure controls pass. Original publication/ACK owners remain until terminal drain. |
| I31 | Which producer bounds/growth/stack charges improve admission? | resolved: retain incomplete-producer envelopes | 48f/F07 | Known flat bounds/growth and constructor/pressure/nested-team/cancellation controls pass. Weighted second/HVP/transpose and provider factories lack a complete scratch-construction maximum; checked result-cell extents cannot replace it. Keep conservative prepayment and actual once-per-team stacks. Reopen a complete producer bound/allocator API or demonstrated admission pressure. |
| I32 | Is remaining glue/thread churn better spawned/isolated/pooled? | resolved: retain existing lanes and scoped placement | 48f/F08 | Thirty-two-point held-cursor run reaches all31 sibling completions, ordered original values and bounded policy visits; existing native lanes/constructor threads and lane-owned capture supply the required overlap. Resource-fixture reruns remain explicitly separate. No new spawn/pool mechanism without a remaining measured bottleneck. Reopen ordinary cold-chunk starvation or repeated construction cost with useful work waiting. |
| I33 | Does a concrete workload justify isolated/concurrent native execution? | resolved: retain native exclusions and one shared process | 48f/F08 | PETSc MPIUNI/SCIP/TPI exclusion, process-wait/CPU-pause, gate pressure/cancellation and negative shared-width controls pass. Existing worker/managed transport lacks complete per-member reconstruction/control/assessment/reap for a concrete isolated occurrence. No generic process pool or unsafe shared-width2 adoption. Reopen an actual reconstructible occurrence with its explicit deployment/IPC owner. |
| I34 | Which bounded transport separates lifecycle from optional history? | resolved: retain prepared/coalesced direct observation | 48g/G04 | Reliable phase retirement is independent of optional-history locking/loss. Held-lock, overflow, unwind/thread-exit, poisoned-state, finite-capacity and terminal controls pass; actual specialization/lowering/materialization intervals close on success/cancellation in the compiler control. Compact/coalesced records remove repeated shared work. Async/multiple producers do not establish SPSC ownership; no ring throughput comparison or scientific observer bottleneck is established. Keep current transport; reopen observed contention/loss that harms required live freshness after these fixes. |
| I35 | Does an allocator change help after materialization fixes? | resolved: retain current allocator | 48g/G06 | Five post-fix ordinary diagnostic observations:10k prepared tuple publications allocate zero;10k real phase+tuple operations allocate60k small objects/1.88MB total but only141B incremental live peak and zero net retention, roughly2.8–3.46ms including counter overhead. Complete setup/snapshot/terminal/drop has zero net counted allocation. No representative scientific allocator bottleneck is established; installed jemalloc remains unmeasured and no allocator-only speed claim follows. Reopen actual post-fix allocation dominance or retained pressure in a scientific workload. |
| I36 | Which remaining question needs a larger composed/longer run? | resolved: no currently justified longer campaign | 48h/H02 | Short work-removal controls support adoption. Reopen on actual unexplained factor/setup growth, retained-memory pressure, bulk-transfer cost or long-horizon work growth;48h records the smallest-case, one-variable,60-second diagnostic route and later-review boundary. |

## Complete recommendation coverage

The [coverage appendix](48-recommendation-coverage.md) maps every register row once to
packages/inquiries or explicit non-adoption. Repeated causes share implementation owners.
Omitted diagnoses remain corrected context/preservation, not hidden tasks. External RC rows
map to the audit's twelve decisions. This coordinator owns all finding/inquiry dispositions;
the appendix has no editable status column.

## Verification and completion

Targeted compile/unit checks and immediate deletion of proven replaced paths accompany each
coherent package. Correctness routes explicitly activate validation. H01 owns completeness;
H02 owns short revealing cases and descriptive work/latency comparison; H03 owns selected
assembled Rust/native/Python/managed journeys, affected integration/parity and full continuing
Clippy/final hygiene/manual feature checks under existing policy. H04 owns enduring publication
and subsequent existing-plan reconciliation. An accepted design/ADR is not implementation.

No numerical speed target, fitted exponent, libc share, reservation/RSS ratio or universal
iteration benefit is a project gate. Source-established amplification needs a correct
replacement without a benchmark. Timing-dependent selection needs matched whole-operation
observations. Exploratory science stays near the existing 60-second cooperative budget
with actual drain; longer work needs I36's specific unresolved question. Keep original failed,
interrupted and repaired acceptance distinct.

Completion requires all AF/N obligations corrected, investigations settled with qualified
selected adoptions implemented, real consumers migrated, replacement leftovers removed,
selected assembled checks passing against zero failures and enduring contracts published.
No-adoption closes an alternative, not a defect. Observable further-investigation triggers
do not imply industrial-capacity qualification or blanket platform implementation.

## Current checkpoint

**Implemented and Tested, complete adopted scope, 2026-10-21.** The independently identified
CA01/C01/AF05 and CA02/E05/I21 gaps are repaired and their settling controls pass. All 53
packages are done, all 22 AF/N findings are resolved, and all 36 inquiries have finite outcomes.
RC01–RC12 and the 162 source recommendation dispositions retain their accepted boundaries.

Conditional boundary preparation now uses relevant instance request/support candidates and
the existing row index, preserving complete original coupling and guard obligations. Public
IDAS attempts establish finite BBD admission before Initial effects; a consumed non-Clone
admission retains the checked configuration/state extent and original execution without a
second charge. Direct preparation, replacement bounds and current stop checks remain.

The combined current-source selection passed 27 tests across math, compiler, native backend
and authored runtime BBD, including all new work-growth/public refusal controls and adjacent
boundary/reset/adjoint/failure regressions. Fresh independent review found no material issue.
Default/no-default and full linked-native/canonical/compiled workspace/all-target Clippy pass
with zero warnings/errors. [48h](48h-qualification-and-design-publication.md#completion-audit-repair-qualification)
records exact commands, conditions and retained qualification limits; earlier passing and
failed selections keep their original scope. H01 consumer reconciliation and H03 affected
qualification are complete; H04 reconciles the final documentation/current-work selection.

Qualified first-principles work reduction remains the adoption preference despite marginal
timings. Practical accuracy stays 0.001. H02/I36 require no automatic longer campaign, and
no full wheel, new platform, global powerset or eliminated environment/provenance proof work
was added. Enduring target contracts remain with their existing architecture/ADR owners;
these repairs implement those contracts without changing them. Other plans retain their
independent scope and status.

## Completion audit

**Audit 2026-10-20; repair closure 2026-10-21 — current conclusion: complete for the accepted adopted scope.**
The independent audit initially found two missing production obligations; the maintainer
then authorized their correction. Both are now **Implemented and Tested** with the targeted
current-source acceptance below; no remaining gap was identified in the audited scope.
The audit baseline was
`bf472ec8ea69ff3f555c00ccfdbb39d04283b4bb` plus the staged, unstaged and untracked Plan48
implementation in the shared checkout, including new generated registry products rather
than only the committed diff. The audit itself was read-only apart from plan corrections;
subsequent repairs changed the four math/compiler/IDAS source owners and their tests.
The original authoring/acceptance descriptions,
authorized amendments and current architecture were read alongside completion claims.
Three fresh read-only reviewers covered A/B, C/D and E/F; the coordinator inspected G/H,
reconciled the source register and independently confirmed both gaps below.

### Coverage and bounded conclusions

All 53 packages, 22 AF/N findings, 36 inquiries and 12 accepted rule boundaries were accounted
for. All 162 source register rows match the coverage appendix exactly once, without missing
or duplicate recommendation labels. That mechanical comparison establishes navigation only;
implementation conclusions come from source and ordinary consumers, including defaults,
public/persisted receiving, feature paths, failure/drain and replacement obligations.

| Accepted scope | Audit conclusion |
| --- | --- |
| A01–A06 | No material gap found in indexed source lookup, mutable views, consumed incremental dependencies or typed masking. Complete checked-artifact persistence and extra source-phase parallelism were conditional; the complete-payload/fresh-check and one-core serial outcomes are legitimate, bounded decisions. |
| B01–B05 | No material gap found in canonical memoization, shared environments/templates, ordinary generated-registry hydration or the one-family typed loader. General/custom builders, current receiving/function eligibility and explicit raw/export fallback remain supported. |
| C01–C08 | CA01's remaining boundary lookup work is repaired and growth-tested. No additional gap found in sparse products, complete-current validity, sparse HVP, bounded named modules, pure-instance reuse, disposable scalar artifacts or the retained realization scope. |
| D01–D07 | No material gap found in native stagnation/refinement meanings, truthful starts, explicit reset eligibility, indexed recycle or qualified portfolio. Fresh Ipopt spaces, explicit grouping/acceleration and absence of a universal backend winner are accepted inquiry outcomes. |
| E01–E08 | CA02's public pre-effect admission is repaired and public-entry/reset tested. No additional gap found in supported IDAS value/forward storage, actual independent coverage, callback/current-product scratch, fixed-trial windows, selected Jacobi, native h0/backward rates, qualified residual fitting or explicit shooting starts. Unsupported backward/Second/BBD storage stays fresh. |
| F01–F09 | No material gap found in native-owner compatibility, event frontier, lane-owned capture/close, preparation-key correction, finite publication handoff, conservative unknown bounds or indexed recovery. Shared numerical policy and eligible local batching do not mean durable native bulk was implemented: independent durable members stay scalar under I29's legitimate no-adoption outcome. |
| G01–G06 | No material production gap found in atomic phase retirement, prepared safe-return callback/compiler tuples, compact KINSOL current/history/durable projection, actual stage scopes or coherent streaming publication. Ring and allocator no-adoptions are bounded; the allocator probe is an ordinary diagnostic fixture, not scientific throughput evidence. Obsolete G Outcome placeholders were corrected during this audit. |
| H01–H04 | H01 reconciles actual repaired consumers; H03's combined 27-test selection and all three full Clippy modes pass; H04 restores completed documentation/current-work state. H02's bounded observations and no-automatic-long-campaign decision remain applicable. Earlier results retain their original conditions. |

The audit does not turn omitted external remedies, conditional no-adoptions or new preferred
designs into mandatory implementation. In particular, it requires no general persistence
platform, universal grouped solve, durable bulk mechanism, ring, allocator change, process
pool, expanded JIT/device realization or new accuracy policy. The original complete-context,
provider/branch, native safety, independent-assessment and acknowledged-publication
constraints remain part of the inspected target.

<a id="ca01"></a>
### CA01 — resolved: selected boundary lookup

**Implemented and Tested; owner 48c/C01; source finding AF05.** The audit found
`CasePlan::visit_binding_dependencies` searching all parent supports after selecting relevant
bindings, and `admit_boundary` scanning parent rows for selected identity/equality checks.
A small late boundary therefore retained work proportional to unrelated parents.

The production support search now visits the selected instance's actual request groups.
`CasePlan::row` uses its existing row-to-ordinal map; the compiler consumes that indexed row
lookup for membership/equality checks. No additional retained index or production lookup
telemetry was added. Complete binding contributions, duplicates, objective/outside-row
coupling, fixed/inactive inputs, guards, source order and unknown-ID refusal remain.

`late_boundary_visits_only_relevant_supports_with_unrelated_parent_growth` and
`conditional_boundary_row_visits_ignore_unrelated_parent_growth` check actual test-only
visit counts with 1/16/64 unrelated bodies/rows. Independent boundary/guard controls retain
acceptance, refusal and connection exemptions. They pass in the 27-test repair selection.
This establishes the intended work removal, without a measured latency or universal speed claim.

<a id="ca02"></a>
### CA02 — resolved: BBD admission before public Initial

**Implemented and Tested; owner 48e/E05; I21/RC05 adopted boundary.** The audit found
`idas::attempt` evaluating `Function::Initial` before its later session construction could
refuse the finite BBD allowance. Direct-session controls had not exercised that wrapper.

An owned, non-Clone `bbd::Admission` is now created before the public attempt's first Initial
evaluation. It binds the checked linear configuration/state extent and owns the original
Execution; session preparation consumes it without charging again and refuses a changed
configuration/extent. Direct preparation/new-session entry points establish their own
admission. Installation/reinitialization retains actual memory and current cancellation
checks; BBD remains outside E01's retained-storage subset.

`bbd_public_entries_refuse_storage_before_initial_effects_and_sample_output` exercises public
fresh/current/workspace paths with absent/insufficient memory or denied storage. It requires
typed refusal, zero Initial/source effects and unchanged caller output. The public finite-
attempt, cancellation-during-admission, configuration-mismatch and failed-reinitialization
controls require exactly one charge and safe failure. They pass with existing current
adjoint/Second/reset and authored Value/JVP BBD controls in the 27-test repair selection.
No equation, solver setting or practical-accuracy change was made.

### Qualification and benefit accounting

**Tested/Measured, historical selected scope:** 48h and package Outcomes retain actual
commands, conditions, initial failures and repaired selections. The audit read the available
native selections (seven and 21 tests), seven parity checks, the Python 20-pass selection
and its exact one-test repair, the runtime 11-pass selection and its exact repair, and the
eight-control native summary, and the
continuing hygiene/repair, three Clippy-mode, feature-canary, governance and documentation
results. Those original selections did not cover CA02's public refusal sequence or CA01's
unrelated-parent growth; the current 27-test repair selection now supplies those controls. No missing historical environment proof,
all-platform result, full powerset, distribution build or all-case campaign is imposed.

Work removal from sparse storage, compiler packing, registry hydration, compatible allocation,
readiness and publication changes retains source/recorded support under its stated limits.
The repair closes CA01's missing boundary-work removal through structural work assertions.
No new timing, whole-process RSS, allocator, industrial-capacity or universal speed claim was
established. New checks exercised the repairs; earlier results were neither erased nor
promoted into coverage of previously unexercised obligations.

**Tested, audit and repair documentation:** `just docs` passed with 548 published chapters;
`document_lifecycle validate` passed with no metadata/scope errors, and its scope projection
reflects the completed native owners. `just turn-end` passed ADR-index and formatting;
`git diff --check` passed. These commands establish documentation consistency, not repaired
production behavior. [48h's repair qualification](48h-qualification-and-design-publication.md#completion-audit-repair-qualification)
owns the separately authorized functional/static repair acceptance.

## Outcome (recorded after implementation)

### What was built

**Implemented and Tested:** all 53 packages deliver the accepted adopted scope. The independent
completion audit's CA01/C01/AF05 and CA02/E05/I21 repairs pass their new controls and relevant
adjacent journeys. H01/H03/H04 reconcile consumers, selected qualification and final records.
All 22 AF/N findings, 36 inquiry outcomes, 162 recommendation mappings and 12 accepted target
boundaries are accounted for in their owning sections.

**Measured, bounded:** earlier selected work counters and observations keep their original
conditions. The repair adds actual bounded-work and pre-effect refusal assertions; it makes
no new measured speedup, industrial-capacity or all-case qualification claim.

### A mistake made and corrected

The completion audit corrected the earlier overstatement of full completion: selected
binding indexes alone had not removed parent support/row searches, and direct-session BBD
denial controls had bypassed the public Initial wrapper. The repairs add actual work-growth
and public-entry refusal assertions, rather than accepting the earlier tests as sufficient.

The initial native preparation hold was released at Cargo startup, although the runner checks
source stability through compilation; it refused before tests. Subsequent holds continue to
actual Nextest startup. Indexed recycle exchange also exposed a production dependency previously
available only to tests, and unmigrated private scientific consumers; the dependency and consumers
were corrected together. Generated registry hydration initially exhausted the compile cap;
sequential outlined producer output removed the compiler blowup before its five controls passed.
Actual fitting population inspection exposed missing distinct foreign coverage across parent,
experiment and profile-chain owners; the production contract and consumers were corrected.
Later control failures included invalid KINSOL box fixtures, decoding the wrong persisted
receipt layer, and interpreting local integration tolerance or scaled stationarity as a
stronger original-output guarantee. Their corrections preserve the actual scientific limits,
positive selected-work checks and original accuracy. Focused checks retain their actual
executed-versus-not-run boundaries; repaired selections do not erase earlier failures.

### Deviations from the plan, deliberate

Qualification used affected selections and a Diffsol-only feature canary rather than a
full Cartesian campaign/global powerset; original failed selections were repaired with
focused reruns. This preserves the authorized design-phase scope and actual evidence limits.
Qualified first-principles changes were deployed without requiring marginal timing wins;
ordinary accuracy stayed unchanged. No full distribution install or eliminated environment/
provenance proof work was reintroduced. Future rule changes follow the decision route above.
