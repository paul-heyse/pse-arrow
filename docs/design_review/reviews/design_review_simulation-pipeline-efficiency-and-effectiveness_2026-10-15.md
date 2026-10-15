---
title: Simulation pipeline efficiency and effectiveness
date: 2026-10-15
tier: design
purpose: target
status: review
doc_retention: while-dependent
standard: Core 3.4; Heuristics for Efficient Architecture 1.0; Process Simulator 1.5; pse-arrow binding
baseline: a08f5fa091db1840fc95b46c4d4f849ce4c5de62 plus the inspected concurrent working tree
---

# Simulation pipeline efficiency and effectiveness

**Decision: Revise.** The simulator has substantial library-owned numerical machinery and meaningful reuse already in place. Its main target gaps are in the composition around that machinery: repeated source/type/function preparation, selected demand arriving after broader construction, cancellation and workspace admission that do not match caller lifetimes, local study admission tied to recorded identity, result interpretation that conflates different response meanings, and metadata reservations whose physical scope exceeds their purpose. A separate numerical scalability defect admits sparse IDAS work against dense state-square allowances. Shooting and horizon composition also discards reusable mathematical/provider workers, and exact transient fitting recomputes an unchanged curvature product on repeated same-point Hessian callbacks.

The best target extends existing authorities and useful lifetimes. It does not add another solver pipeline, a second generic model, a parallel cache hierarchy or a universal execution planner. Numerical improvement has a distinct second axis: reduce actual integrations, derivative expansion, factorization and rejected work where the requested product and library lifecycle permit. Same-point trajectory/checkpoint products, exact curvature reuse, derivative actions and selected compiled/batched evaluation are valuable directions, with explicit capability and memory tradeoffs. They are not measured speedups or excuses to weaken original physical assessment.

## Scope, basis and evidence — slots 1 and 10

This is a full simulation-pipeline target review, from source admission through preparation, class routing, evaluation, native execution, workflow composition, publication and result consumption. It covers interactive value/structure edits; independent, causal and lazy studies; recycles/initialization/continuation; sensitivities and fitting; dynamics, events and shooting/horizon contexts; Rust/Python entry points; local, native-child and managed execution where their contracts interact. Development dependencies are assessed where these responsibilities must be changed or tested independently. Unrelated repository-wide infrastructure and build optimization are excluded.

The only judgment criteria are the selected standard and the functional target in the Process Simulator profile and blueprint §0/§0.2. ADRs, plans, policies, supported-profile limits and pins describe the current design and can be changed through the rule impacts below. Current mechanisms are not immutable constraints on the target. No library or license was rejected because of absent consumers or licensing. Eliminated environment/provenance-proof work is not reinstated; scientific source/result identity, physical correctness and actual cancellation/drain remain required.

The coordinator conducted the preceding [Plan39 simulation audit](../../plans/39d-qualification-and-plan38-cutover.md#simulation-audit--2026-10-15). Independent scoped preparation and numerical reviewers, a library researcher, and a fresh integrated assessor examined the current dirty source. Their [supporting inquiries](../evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/README.md) distinguish facts, proposals and coverage limits. The coordinator read decisive source and reconciled the recommendations. The original audit remains unchanged; new recommendations await future adoption and are not implementation authorization.

**Interface-checked:** current source, exact installed/vendored/forked library interfaces and their callers. **Measured / Tested, prior dated evidence only:** the named audit observations and selected controls retain their original conditions. **Proposed:** every correction and performance benefit in this review. No new product build, simulation, numerical test, benchmark or host probe ran during this review. Static evidence suffices for the demonstrated amplification and ownership defects; it does not establish their speedup or a globally optimal numerical strategy.

The interrupted 32-block cold case spent approximately 36.47 seconds in source admission and 480.52 seconds preparing, with about 518.73 seconds total wall time. It never reached a numerical RunHandle. Its bounded CPU captures have partial source coverage and unknown sampling loss. Neither those frames nor the older Criterion estimate attributes minutes to nonlinear convergence. The scalar sweep's 234 joins are a useful prefix, not completion of 1,000 checked points. The managed retry completed sixteen usable/fresh one-attempt outcomes, then failed before its persisted numerical/native-metric checks. Three cold-dependent positions were not run. Completed scalar-reuse, fitting and shooting controls and selected native/Python journeys are positive evidence at their named scope, not whole-product qualification.

## Responsibilities and preserved strengths — slots 2 and 3

The intended division remains coherent: authored physics and lexical meaning; finite specialization and structural preparation; library integration and native solving; workflow composition; scientific assessment; and retention/transport each have their own decisions. Some physical lifetimes currently cut across those responsibilities in expensive ways.

| Responsibility and current owners | Contract to preserve | Target locality |
|---|---|---|
| Selected source and checked package: modeling canonical index, `pse-modeling` resolver/checker | Visibility, import precedence, all candidates, Pending/absence, current physical/type context | Faster access and borrowed overlays without a second lexical/type authority |
| Compiler/Salsa/math preparation | Complete dependencies, pure immutable products, stable body identity and one mutable compiler writer | Demand-shaped preparation, shared immutable inputs, cancellable serialization before active dispatch |
| Class/policy/native adapters | Derived class, contextual capability, explicit method/settings, library-owned iteration and factors | One routed pipeline; backend integration absorbs changes rather than consumers branching on handles |
| Attempts and workflows | Fresh controls, explicit starts/branches, occurrence identity, original clocks, private mutable workers and real join | Reuse compatible neutral workers and same-point products at actual operation lifetime |
| Scientific outcomes and analysis | Native status is not usability; physical checks, rank/validity and withholding have owners | Family-qualified response meaning; common storage need not imply common certification |
| Canonical transport/publication/read | Complete requested output, exact ACK/manifest, protected source/file lifetime, honest partiality | Bound and group actual metadata work separately from file payload work |

Already implemented strengths constrain the remedies: multi-output Symbolica evaluation/CSE, first directional products, tracked compiler phases, exact-point achieved-demand caches, faer symbolic/numeric separation and multi-RHS solves, KLU guarded refactor/fresh fallback, retained/reset native sessions, typed class/contextual routing, independent original-space assessment, native KINSOL recycle convergence, bounded fit experiment workers, and genuine native join ownership. Adding these again would create machinery rather than improve the design. Required coupled/global checks and conservative bounds for genuinely opaque native fill remain legitimate.

### Physical and numerical meanings

This table records consumed contracts, not a new validation of every scientific model.

| Quantity/product | Units, basis and conventions | Validity/authority preserved |
|---|---|---|
| Authored states, parameters, equations and observations | Declared quantity type, canonical units, basis/reference and time origin; native coordinates have explicit scale/offset | Authored physical definitions and selected physical context; bounds, envelopes, guards and balance closure survive rewriting/reuse |
| Residuals, Jacobians, Hessians and derivative actions | Declared derivative order/source and original-to-native coordinate mapping; scaled solver criteria remain distinct from physical acceptance | Symbolica/provider contracts and independent original assessment; missing order is not zero and finite differences are explicit |
| Transient response | Physical output/parameter units and exact time/mode/event/observation scope | Consistent initialization, supported smooth sensitivity route, actual derivative history and explicit withholding; not a KKT response by default |
| Root/optimal response and uncertainty | Original coordinates, selected branch/active set and statistical/model assumptions | Root rank/neighborhood or KKT/curvature validity as applicable; a trajectory or graph path supplies neither automatically |
| Diagnostic progress | Native iterations, evaluations, steps, factors, wall/CPU time and reservations retain different units/scopes | Passive observations, actual attempt/generation, reset, overlap, loss and partiality; no extra scientific evaluation for inspection |

**Well-posedness:** the inspected source derives structural/class facts before dispatch, separates stream topology from numerical incidence/solve order, and uses matching/presolve and typed capability admission. Dynamic roles and consistent initialization are explicit within the current ODE/index-1 profile. These are meaningful foundations, not new evidence for every model's rank, envelope or convergence. General/higher-index DAEs and other unsupported combinations are not silently accepted or promised by this review.

## Revealing scenarios — slot 4

| ID | Trigger and distinguishing obligation |
|---|---|
| <a id="s01"></a>S01 | Value, equation, import-membership and physical-context edits: retained structure agrees with fresh preparation; absent/Pending and evolving overlays stay correct. |
| <a id="s02"></a>S02 | Cancel during source resolution, specialization or workspace admission; two waiters share a key but have different clocks. One caller cannot govern another's valid work, and issued work stays charged through drain. |
| <a id="s03"></a>S03 | Public local/durable explicit or lazy physical study: common scientific admission, honest ownership, explicit publication and distinct occurrence results. |
| <a id="s04"></a>S04 | Growing scalar durable sweep and complete managed result read: protocol work follows useful retention demand; metadata and payload lifetimes fit the declared operation. |
| <a id="s05"></a>S05 | Root, optimal and dynamic responses feed graph/artifact consumers, including withheld/partial cases: interpretation and certification match the actual family. |
| <a id="s06"></a>S06 | Large diagonal/banded and genuinely dense IDAS problems with direct and Krylov methods: sparse preparation follows complete support, while actual factors/preconditioners retain appropriate finite bounds. |
| <a id="s07"></a>S07 | Repeated fit/shooting callbacks at the same and changed point, with value/forward/adjoint/curvature demands: reuse only recorded capability; preserve fresh-IVP semantics when the problem changes. |
| <a id="s08"></a>S08 | Multiple-shooting trials and rolling-horizon periods: retain compatible workers; parallelize only genuinely independent anchored windows, preserving causal starts/feedback and ordered reduction. |
| <a id="s09"></a>S09 | Nonconvergent recycle, failed initialization, changed scaling/branch/profile and unsupported solver method: native policy owns iteration, original specification survives and failures remain attributable. |
| <a id="s10"></a>S10 | Add a unit/property or replace a solver/transport implementation; test admission locally: semantic additions stay local and tests use actual dependencies. |

## Findings and corrective contracts — slot 7

Source details and adversarial cases are in the supporting documents. Each finding below is source-backed; remedies remain Proposed. Priorities reflect consequences, not measured fractions of total runtime.

### <a id="f01"></a>F01 — Import lookup ignores its existing root index

**AP-07; DP-10; G9; S01/S10. High.** `SourceIndex::insert` maintains named root buckets, but `imported` scans all selected rows for each matching root import after its completeness checks. Import-heavy acquisition repeatedly examines unrelated declarations.

Use that existing index after the same completeness premise. Preserve every candidate, import-group precedence and declaration-ID ordering: acquisition order is not interchangeable with the old scan order. Verification compares complete lookup answers under reverse insertion order, guards, duplicate candidates and Pending-to-found transitions, plus examined-candidate work. No new cache or numerical solver is needed.

### <a id="f02"></a>F02 — Stable lexical type context is rebuilt inside specialization

**AP-07; DP-09/10; G9; S01/S07. High.** `named_types_with` already uses prefix access, so the defect is not a fresh whole-repository scan. Arithmetic typing, constructor argument loops and `Engine::source_types` nevertheless repeatedly resolve/materialize broad visible type environments over an unchanged borrowed package.

Retain a lexical-owner base within immutable specialization and apply current arguments, values, optional refinements, reader context and synthetic-function types as overlays. Hoist unchanged constructor lookup; keep per-argument checking. Mutable admission is excluded from that reuse scope. Preserve mutation, prefix/shadowing, lowered-function and refinement controls; add same-owner/different-overlay cases. Construction counts can show removed rebuilding without a latency claim.

### <a id="f03"></a>F03 — A function override clones the complete checked package

**AP-01/07; DP-10/16; G9; S01/S10. Medium.** `SpecializedModel::function_contracts` copies declarations, names, children, types and other package inventories to update only synthesized functions/lowered-function contracts. Repeated contextual and contribution admission pays for unrelated immutable state.

Make admission consume the original package plus an owned evolving function overlay through the same lookup authority. Preserve functions added after first use and reconstruction-local result transformations; neither a second name registry nor a stale augmented-package cache is a correction. Verify equivalent inference/guards/provenance and isolation of reconstruction overrides, then demonstrate removal of full package copies.

### <a id="f04"></a>F04 — Selected function preparation expands broader work first

**AP-03/07; DP-08/10; G9; S01/S07. High.** `prepare_modeling_functions` builds a full value CasePlan before selecting requested rows/coordinates. Observation-program lookup constructs selected structure, then a miss reconstructs it. Fitting, dynamics and checks inherit those costs.

Construct selected products directly from admitted inventory, or reuse a genuinely needed full plan; carry an already-built observation structure across lookup/construction. Preserve complete contribution sums, requested coordinate order, bounds, native forms, guards and provider demand. Opaque coupled evaluation remains broad where its supplier requires it. Compare selected versus current assembly across permutations, derivatives and original checks before migrating the shared consumers.

### <a id="f05"></a>F05 — Grouped outputs rebuild a projection-level lookup

**AP-07; DP-10; G9; S01. Medium.** Grouped `normalize` reconstructs the complete runtime/formal map for each output, although it is stable within the grouped plan. Existing transitive local-member trimming is already useful.

Build the projection map once and borrow it per consumer. Keep expression-local Let state, fresh renaming, dependency-order numbering, validity-only dependencies and checked-member attribution separate. Verify overlapping but distinct closures and equal normalized bodies across different instance identities; one global renamer would be unsound.

### <a id="f06"></a>F06 — Child preparation cancellation is disconnected from its parent

**AP-05/07; DP-19/20; G9; S02. Critical prerequisite.** `child_with_deadline` retains the effective clock but substitutes a fresh token. Ordinary checked selection and engagement do not bridge parent cancellation, explaining why the audit stop request did not promptly reach specialization.

Use hierarchical cancellation with upward/sibling isolation. Independently track parent handling in ephemeral-study coordination: its current `!child_cancelled` guards also repopulate the frontier and would stop doing so after token propagation is fixed. Preserve interruption cause, original deadline and actual issued-work join. Exercise public preparation, already-cancelled parents, siblings, blocked study points and last-waiter drain; the passing managed cancellation control does not close this path.

### <a id="f07"></a>F07 — Shared preparation inherits the creating waiter's deadline

**AP-03/05/07; DP-09/19/20; G9; S02. High.** Keyed frontier/completion flights correctly share mathematical identity without caller clocks, but their producer captures the first waiter's deadline. A later, still-valid waiter can receive a producer timeout caused by another caller.

Separate caller clocks from shared-producer scheduling. Prefer a finite service-owned producer policy with each waiter retaining its original deadline and last-waiter retirement. If that policy cannot fit a consumed contract, use execution-clock compatibility separate from scientific identity, or an explicit joined-expiry retry without renewing caller time. Verify short-A/long-B, reverse order, explicit cancellation, all waiters leaving and no duplicate active producer before join. Merely passing cancellation-only sharing tests is insufficient.

### <a id="f08"></a>F08 — Compiler exclusion is acquired after scarce worker dispatch

**AP-05/06/07; DP-19/20; G9; S02. High.** Charged native preparation workers can block on the compiler workspace's synchronous mutex behind another preparation. Token signalling cannot interrupt that lock acquisition; correct join ownership then waits for unrelated work. This is source-inferred concurrent behavior, not attribution of contention to the single cold audit case.

Make workspace serialization cancellable before active CPU/native dispatch, then transfer exclusive preparation authority to the worker. Preserve one writer, atomic head comparison/use, branch/historical isolation and coherent publication. Making the lock async inside an already charged worker misses the resource coupling. A gated A/B test must show cancelled B leaves the admission queue without waiting for A or consuming an active worker, while A and subsequent head use remain correct.

### <a id="f09"></a>F09 — Public local study admission requires recorded source identity

**AP-03/04/06; DP-05/18; G9; S03/S10. High.** Public study admission always takes `recorded_revision` and canonical physical-source publication, although the local package/executor can own the needed admitted science without a recording sink. The public ephemeral physical study fails before any point executes; lower fixtures with fabricated receipts do not qualify this capability.

Own study intent and admitted model/physical context independently of retention placement. A local execution retains genuine immutable source owners; a durable descriptor obtains recorded handles through explicit publication. Rust/Python callers must receive the corresponding honest ownership form through shared scientific admission. Verify local physical studies without a store, explicit durable refusal/publication, lazy selected admission, occurrence ordering and source lifetime. RC01 identifies the current rule impact.

### <a id="f10"></a>F10 — Result analysis treats all responses as scalar local analysis

**AP-02/04; DP-02/08/21; PS-12; G9; S05. High.** Result-analysis dispatch unconditionally promotes LocalAnalysis, whose admission requires scalar local validity. Dynamic forward responses do not supply that product, so both audit result cases fail before graph construction.

Dispatch on admitted scientific response meaning. Use existing trajectory evidence for integrated response/time scope and local-analysis evidence for root/optimal products where their promises fit. Trajectory artifacts already retain response rows, so a new kind is not presumed. Keep certification/withholding with the producing scientific owner; do not invent KKT verdicts or remove the LocalAnalysis guard. Verify valid and withheld root, optimal and dynamic cases, partial/event trajectories, graph evidence coordinates and independent retention after producer/graph retirement. RC02 bounds any new persistent validity meaning.

### <a id="f11"></a>F11 — Metadata allowances inherit physical file maxima and overlap

**AP-07; DP-10/19/20; G9; S04. High.** Result selection retains a 96 MiB temporary response allowance while snapshot metadata requests another 96 MiB; file-open and analysis/artifact metadata use similar file-shaped allowances. The managed observer refuses its first persisted read with two real compiler leases still present. Metadata pages have bounded descriptor contents and separate payload-decode grants.

Separate temporary RPC, retained metadata, snapshot inventory and file decode ownership. Transfer exact continuing metadata, retire/reuse genuinely drained temporary grants and derive safe decoded metadata bounds from their own contract. Keep SDK cancellation suffix ownership and protected read tickets. Pressure coordination may release demonstrably idle optional retention, but a cap bump or deleted accounting is not this correction. Verify nonempty paging/file-open under the intended observer workload, concurrent/cancelled RPC suffixes, exact retained bytes and eventual release. Empty `0..0` controls bypass the defect.

### <a id="f12"></a>F12 — Sparse IDAS admission is charged as dense state-square work

**AP-07; DP-10/20; PS-11; G9; S06. High.** `native_scatter_bytes` computes a state-square hash-map allowance before querying complete sparse support. IDAS then builds the actual sparse union. Consistent initialization additionally charges dense KLU fill for SPGMR/SPFGMR routes that do not select that factor.

Prepare and reuse complete all-mode sparse layout/scatter metadata and charge known containers/construction overlap. Direct factors retain their own finite opaque-fill allowance; Krylov/preconditioner storage follows the selected method. A compact column lookup is an alternative to the hash map, not a mandatory new representation. Preserve transpose/diagonal/second-order mappings and out-of-pattern refusal. Verify sparse-growth admission and actual Krylov demand, genuinely dense refusal, changed support and finite charge through native drain. Input nonzeros do not bound direct-factor fill.

### <a id="f13"></a>F13 — Window composition reconstructs compatible mathematical workers

**AP-03/07; DP-10/19/20; PS-11; G9; S07/S08. Medium/high by workload.** Shared window integration/gradient helpers construct DynamicWorker/Anchored state on every shooting trial/window and horizon period. The composed workflow retains reports rather than those compatible workers. Existing fitting experiment workers show a viable ownership seam.

Retain attempt-owned, owner-affine window workers, rebinding values, anchors, time and execution scope and resetting invalid trial/native state. A horizon plant can retain one worker on its staged thread. Independent anchored windows within a shooting trial may use finite admitted waves with ordered reduction when the working set warrants it; causal initial-point construction and feedback remain serial. Verify construction counts separately from integrations, retained/fresh numerical equivalence, changed scope/provider/IVP, failure/cancellation and real teardown. Neutral worker reuse does not authorize hidden multistep continuation across changed IVPs.

### <a id="f14"></a>F14 — Exact transient fit curvature is recomputed for same-point Hessians

**AP-07; DP-09/10; PS-11; G9; S07. Medium/high for exact transient fitting.** `FitOracle::hessian` calls `transient_curvatures(x)` on every exact-mode callback. The existing Point retains predictions/response/Jacobian products, but not this completed transient curvature. Native objective weight is applied afterward; changed solver weights/multipliers at the same scientific point need not recompute the underlying transient product.

Retain that curvature at the existing exact-point owner, keyed on the experiment, observations/residual weights, directions, provider/profile/mode/schedule/start and accuracy premises that actually determine it. Recombine native weights afterward; atomically replace complete products and release failed/old points. Changed observation weights do affect the residual-curvature block, unlike the outer objective multiplier. Verify repeated same-x callbacks with changed native weights avoid another integration, while changed scientific weights/context invalidate; preserve independent final original assessment as a distinct mode. Frequency and benefit in representative optimizer runs remain unmeasured. This narrower finding does not require a native checkpoint redesign.

## Additional opportunities and consequential investigations — slots 8 and 9

These directions have plausible source/interface support but are not established numerical bottlenecks, selected defaults or required new mechanisms. Each has a concrete decision owner and settling question.

| ID / owner | Candidate and comparison | Preservation and settling evidence |
|---|---|---|
| O01 — integrated fitting/shooting owner | Retain a same-point capability frontier: completed primal products, eligible native checkpoint path, selected tangents, adjoints and related derivative products. F14 separately addresses completed exact residual-curvature reuse without requiring checkpoints. Compare demanded replay, checkpointed primal and eager first order. Existing exact-point achieved-demand caches already avoid many repeated callbacks. | Changed point/profile/provider/mode/schedule/start/accuracy invalidates capability. IDAS primal-only history cannot retroactively supply missing tangent history; record required capability upfront or replay. Curvature reweighting at unchanged context need not rerun integration. Observe actual callback orders, upgrades, integrations/replay and peak retained state before choosing policy. |
| O02 — durable publication/transport owner | Group compatible metadata finalization, empty memberships and already acknowledged immutable metadata while keeping each required IPC schema separate. Operation-scoped protection may avoid repeated source admission if its permission really spans the work. | The sweep is durable, not pure local edit/solve. Preserve current fences, complete demanded outputs, exact ACK/manifest, retry identity and source/file lifetime. Reconcile the failed append-queue test: table completion currently flushes, and absence of all prior ACKs is not a sound oracle. Select bounded metadata batches only after distinguishing actual protocol phases. |
| O03 — implicit/provider/math integration owner | Use existing first directional products, transpose/multi-RHS factors and qualified selected second contractions when consumers need actions rather than full jets. Full residual second-order expansion can exceed the selected action's scope. | Preserve IFT chain rule, Taylor/factorial conventions, all required support/guards and provider derivative order. A full requested dense output still has its output cost. Establish an action-capable supplier contract before narrowing opaque evaluation; compare actions with independently perturbed originals. |
| O04 — Symbolica integration owner | Qualify compiled/JIT/SIMD/batched evaluation for repeated compatible workloads after removing avoidable whole work. Exact 3.0.1 has relevant paths, but `native_code_generation` is absent from the current feature profile. | No assumption of no-FMA, determinism, cancellation, provider error or ISA compatibility. Include compilation amortization, executable/scratch ownership and lane/tail behavior. This is a bounded candidate, not a reason to enable a feature globally or change tolerances; RC03 governs adoption. |
| O05 — class/routing owner | Compare a direct library linear route for proved affine square roots with existing KINSOL, and carry existing neutral native preparation across real study lifetimes. | Preserve original bounds, scaling, class proof and per-state assessment; linear algebra must not silently solve a different feasibility problem. Existing native reset/refactor/update paths are already substantial. Unmeasured small-case benefit does not establish a new mandatory solver. |
| O06 — presolve/native owner | Compare native original-reaction provenance work with an explicitly primal-only tightening route when demanded output permits withholding dual/KKT quantities. | The pinned POUNCE fork has a real bound-tightening/FBBT provenance gap. Affine elimination recovery and SCIP global bounds do not repair it. Current FBBT infeasibility prechecks remain useful. No full-reaction route is established by this review; RC04 applies to a narrower output contract. |
| O07 — shared runtime/resource owners | Coordinate pressure on genuinely idle retained compiler/native/cache state across math and result consumers after F11 removes oversized overlaps. | Math already trims idle lanes; active borrowers and useful warm reuse stay protected. Fixed opaque workspace bounds are not shown shrinkable from names or sampled RSS. Use the existing resource/retention owners before inventing another manager. |
| O08 — operation/context diagnostic owners | Add bounded responsibility counters for lookup/environment/overlay construction, selected work, integration upgrades/replay, map/factor construction, cancellation stages and active-owner relationships. | Observe actual work without new evaluations, policy duplication or unbounded metric names. Root-link eviction, managed process scope, stale checkpoints and loss remain explicit. Counts establish removed work; inclusive phases are not exclusive CPU totals. |

The [library matrix](../evidence/simulation-pipeline-efficiency-and-effectiveness-2026-10-15/library-fit.md) compares current exposed capabilities, available-but-unexposed mechanisms, feature-qualified candidates and unsupported/unknown contracts. Exact installed/vendored source governs this checkout; Context7 current-main documentation is discovery, not cross-version proof. Symbolica 3.0.1 differs from the shared skill's 3.0.0 baseline. The exact POUNCE fork, not arbitrary upstream 0.12.0, governs its reset and provenance behavior. No second modeling wrapper, new solver family or library upgrade was selected without a concrete consumed gap.

## Numerical composition and verification — slots 5 and 10

| Stage / owned operation | Formulation, derivatives and scaling | Class/library capability and outcome | Work/validity obligation |
|---|---|---|---|
| Selected preparation | Authored guards, physical context, ordered selected rows/coordinates and declared derivative demand | Compiler/Salsa with Symbolica/Numerica integration; complete/Pending/refused stay distinct | Reuse immutable premises, not mutable admission overlays; F01–F05 preserve exact assembly. |
| Steady/root/NLP/coefficient/cone/discrete solve | Derived class, admitted representation, complete derivative order and physical-to-native scaling | Existing class-specific native adapters and one routed execution lifecycle | Library owns iteration/globalization/factors. Effective method/settings and independent original-space residual/bound/domain/closure checks remain separate. |
| Initialization/recycle/continuation | Explicit temporary overlays, valid map/tear definitions, branches, finite stages and physical acceptance | Native root/corrector/optimization operations under domain composition | Failed stages leave specification intact; frozen inner-map semantics cannot be changed into continuation merely for speed. |
| Dynamics/IC/events | Declared time/mass roles, all-mode support, consistent start, event/reset/scheduled changes and selected derivative order | Diffsol/IDAS contracts; typed convergence/limit/evaluation/cancellation and trajectory coverage | Sparse admission must match selected native storage. Accepted primal integration does not manufacture sensitivity history. |
| Fit/shooting/sensitivity | Same definitions, experiment/observation identity, derivative actions, exact/Gauss–Newton and root/KKT assumptions | Existing native NLP/IVP/factor operations; family-specific response/uncertainty validity | Retain compatible workers and point capabilities; changed IVPs reset; final original checks remain independent. |
| Publication/read/analysis/diagnostics | Units, original coordinates, scientific interpretation and exact evidence references | Canonical ACK/manifest/ticket and typed artifacts; passive diagnostics | Metadata is not file payload; provisional/partial/cancelled is not committed/qualified. Actual drain ends ownership. |

Recommended later verification is risk-directed, not a mandatory Cartesian campaign. Pure lexical, overlay, assembly, scheduling and reservation controls use only actual dependencies. Numerical equivalence needs representative nontrivial perturbations and original checks; public local/durable and managed journeys cover the real boundaries. Sparse-growth and workload-construction counters can establish eliminated work without repeated timing. A numerical speedup/convergence-superiority claim would require its own controlled complete-operation evidence, which is not supplied here.

No new unit/property conformance or clean-room IDAES parity was executed. Existing audit passes/failures retain their named conditions. General DAE extensions, hybrid sensitivity support, arbitrary solver combinations, GPU/MPI/distributed single solves, server-loss recovery, statistical timing and broad hygiene are unexamined, not silently passed or newly required. This review neither revives the eliminated environment-proof work nor claims completed Plan39/40 qualification.

## Audit reconciliation and independent challenge

| Audit item | Position in this review | Corrective boundary / evidence limit |
|---|---|---|
| AUD39-01 | Confirm, refine ordering and scan scope — F01 | Existing indexed buckets remove unrelated examination; preserve completeness and old candidate order. Inherited-member caching remains conditional. |
| AUD39-02 | Confirm, refine reuse scope — F02–F05 | Prefix lookup already exists. Reuse immutable lexical context, not mutable admission or evolving overlays. Broader package/plan/output rebuilding has distinct closure obligations. |
| AUD39-03 | Confirm and broaden — F06–F08 | Hierarchical tokens need simultaneous study-bookkeeping repair. Separate shared-producer clocks and pre-dispatch compiler admission are additional defects, not measured causes of the single cold delay. |
| AUD39-04 | Confirm — F09 / RC01 | Public source admission, not the lower ephemeral executor, blocks local physical studies. Fake receipts or compulsory store initialization do not solve it. |
| AUD39-05 | Retain as bounded investigation — O02 | Existing compatible block batching, demand shaping and original completion are strengths. Whole-operation query counts do not prove a publication bottleneck. The mixed-relation queue oracle conflicts with current table-completion flush semantics. |
| AUD39-06 | Confirm and refine — F10 / conditional RC02 | Use scientific response family; existing trajectory storage may suffice. Preserve scalar validity admission and distinguish recorded evidence from stronger certification. |
| AUD39-07 | Confirm and broaden response scope — F11 / O07 | Metadata/page/file-open allowances need their own bounds and actual ownership transitions. Real compiler leases are not proven duplicates; host OOM and arbitrary capacity increase are unsupported diagnoses. |

The fresh assessor independently confirmed the integrated architectural deficiencies and rejected interpreting safe artifact/resource refusals as false numerical success. It also identified exact transient curvature as a source-established repeated product, while the numerical/library inquiries initially grouped it with broader same-point retention opportunities. The coordinator reread `hessian`, `transient_curvatures` and the later objective weighting and promoted this narrower obligation to **F14**. This changes neither the unknown callback frequency nor the Proposed benefit. O01 remains a conditional native-history strategy; same-point value-to-first-order replay alone is not a defect because avoiding unused sensitivities at rejected trials has a real benefit.

Independent judgments also constrained the target: no missing generic Newton/globalization/factorization engine was established; no existing factor/reset facility is claimed wholly absent; FBBT's provenance gap is real; native reports are not checkpoints; the 256 MiB pool failure does not establish redundant workspaces; and optional window parallelism is not needed to close F13's worker-reconstruction defect. Supporting local judgments do not independently accept their interactions.

## Architectural and behavioral judgments — slot 6

| Foundation | Verdict for this target boundary | Decisive scenario and consequence |
|---|---|---|
| AP-01 separation | Violated at the local-study boundary; useful separation elsewhere | S03/S10: scientific admission requires recorded placement (F09), propagating infrastructure into local work and tests. |
| AP-02 stable consumed contracts | Violated in response composition | S05: a shared row shape does not supply the certification demanded by the generic consumer (F10). Existing solver capability/adapter separation remains useful. |
| AP-03 composition | Violated | S03/S05/S08: local study, trajectory analysis and reusable window execution cannot compose through the meanings/lifetimes already owned (F09/F10/F13). |
| AP-04 adequate governing domain model | Violated for response-family admission | S05: transient, root and optimal response conditions differ, but generic analysis admits through one scalar local-analysis meaning (F10). Its current refusal is protective, not fabricated certification. |
| AP-05 explicit enforceable structure | Violated in caller/producer/workspace lifetimes | S02: child scope, shared clock and serialization admission do not enforce intended caller isolation and stopping (F06–F08). |
| AP-06 local reasoning/testability | Violated for public local-study admission | S03/S10: semantic local admission requires an unrelated recording owner (F09). Pure compiler/policy controls and necessary native/durable integration remain appropriate. |
| AP-07 execution fit | Violated | S01/S02/S04/S06–S08: demonstrated repeated preparation/products, resource coupling, dense sparse-admission and oversized metadata overlap (F01–F08/F11–F14). Static amplification needs no claimed numerical speedup. |

| Gate | Judgment at the reviewed breadth | Evidence / limitation |
|---|---|---|
| G1 Authority | Unresolved for complete pipeline; inspected authorities supported | No competing writable scientific authority established. Derived copies can amplify work without being separate mutable meaning. |
| G2 Semantic fidelity | Unresolved; observed artifact refusal is protective | F10 currently refuses mismatched certification rather than silently accepting a reinterpreted derivative. Corrected family translation needs explicit scope. |
| G3 Validity | Unresolved; scoped enforcement supported | Physical, structural, domain, capability and resource boundaries are present; all model/provider combinations were not qualified. |
| G4 Hidden behavior | Unresolved at complete breadth; no new undeclared pure effect established | Unwanted explicit publication in F09 is a dependency defect, not automatically an undisclosed inspection effect. Diagnostic reads must stay passive. |
| G5 Consistency/recovery | Unresolved; ACK/ticket/join strengths retained | No false committed prefix demonstrated. Source cancellation defects and untested recovery interactions still limit full acceptance. |
| G6 Transformation/reuse | Unresolved; scoped positive reuse evidence retained | Scalar reuse and complete-dependency contracts exist. Proposed overlay, selected-plan, curvature and checkpoint remedies need their respective equivalence controls. |
| G7 Truthful capability | Unresolved for full supported family | Public study/result paths have functional blockers, while unsupported/refused outcomes remain honest. No silent solver fallback or false successful output established. |
| G8 Library leverage | Pass for examined generic numerical/structural mechanisms; other breadth unresolved | Established libraries own iteration, globalization, factors, matching and integration. The defects do not justify another generic solver/model engine. |
| G9 Architectural fitness | **Fail** | Each violated foundation above has its own scenario/consequence; local correctness cannot average them away. |
| PS-G1 Physical consistency | Unresolved | Physical/context/envelope/closure owners exist; every unit/property/boundary is not validated by this review. |
| PS-G2 Well-posedness | Unresolved at complete supported breadth | Class-aware structural admission is real; numerical rank and complete dynamic consistency require their own evidence. |
| PS-G3 Numerical integrity | Unresolved | Independent original assessment and typed outcomes are strengths; complete derivative/event/provider fidelity is not established by source samples or the interrupted cold case. |

**Architectural fitness fails; complete behavioral/scientific adequacy remains unresolved.** The target cannot be accepted, but neither lack of comprehensive qualification nor a safe capacity refusal alone is a newly demonstrated physical error.

## Rule impacts and follow-up ownership — slot 11

These are recommendations/decision routes, not applied amendments. Original AUD39 status remains in 39d. New F01–F14 and O01–O08 await a future authorized plan-creation step; the responsible modules named above are proposed work owners, not scheduled packets. No active plan is superseded or expanded by publication.

| ID | Current rule/record | Recommended change and dependencies | If the current rule is retained |
|---|---|---|---|
| <a id="rc01"></a>RC01 | Current recorded `StudyDefinition`/public study-source contract, blueprint §19.3/§21 and the canonical-source clause in §0.5/§21.1 | F09 requires an honest runtime-local admitted form alongside the portable recorded descriptor, common scientific/occurrence policy, and explicit publication for durable use. Evolve affected public Rust/Python and durable document contracts deliberately. | Recorded descriptors remain valid for durable use, but keeping them as the sole local admission form leaves F09 and the target limitation unresolved. |
| <a id="rc02"></a>RC02 — conditional semantic extension | Current LocalAnalysis promise/admission and generic response-analysis realization, blueprint §19.2/§20.4 | F10 must select existing evidence by family. Reusing Trajectory within its existing promise is a dispatcher correction, not automatic new artifact/schema authority. A stronger/new transient response certification or persistent family requires an explicit contract/schema amendment before reliance. | Keep LocalAnalysis for its valid scope. Existing trajectory provenance can still be used with its weaker declared meaning; stronger new certification remains unavailable. Never weaken the scalar validity check to make the failing benchmark pass. |
| <a id="rc03"></a>RC03 — conditional capability expansion | Current interpreted feature profile and JIT/SIMD exclusion in blueprint §25; current floating-point policy | O04 may qualify an explicit compiled/batched evaluator route and update supported-profile/feature declarations if its full consumed contract fits. No dependency or feature change is installed by this review. | Keep the interpreter and existing multi-output/directional evaluation; F01–F14 do not depend on compiled execution. An unresolved floating-point or lifecycle mismatch prevents adoption. |
| <a id="rc04"></a>RC04 — conditional output/presolve policy | Current original-reaction eligibility and disabled LinearBounds/FBBT execution, blueprint §15.3–§15.5/§18 and the exact POUNCE fork | O06 can investigate native reaction provenance, or explicitly select a primal-only tightening contract with unavailable dual/KKT quantities represented honestly. General reaction recovery is not claimed. | Preserve current restrictions and useful infeasibility prechecks. No primal-only path is silently inferred and no full-dual capability is invented. |

No mandatory rule change was found for indexed access, borrowed overlays, selected preparation, hierarchical cancellation, cancellable compiler admission, exact metadata transfer, sparse method-aware bounds, neutral worker reuse or same-point completed curvature caching. They correct realization while preserving current guarantees. Shared-producer policy needs explicit lifetime clarification without renewing original caller clocks or placing deadlines into scientific identity. Same-point history does not change ADR-0186's fresh-IVP rule for **changed** trials; if a future policy requires a fresh forward pass on every callback, narrowing that extra policy is a separate concrete impact. Changed-trial multistep reuse, portable native checkpoints, altered physical acceptance and weakened drain/protection are not recommended here.

Ordinary dependency changes remain eligible, but a dependency/API/version name does not establish current availability or a measured benefit. No ADR acceptance, policy change, library upgrade, implementation or compatibility layer is authorized by this review.

## Priorities, prerequisites and final decision — slot 12

Priorities follow consequence; prerequisite order follows the meaning needed by consumers:

1. **Reliable stopping and caller isolation:** F06 before long dependent reruns; F07/F08 before claiming sound concurrent preparation. Faster preparation does not repair cancellation or shared-clock ownership.
2. **Usable public composition and resource lifetimes:** settle F09's local/recorded boundary, F10's response meaning and F11's allocation/transport transition before changing public contracts, artifact oracles or reservations. These are independent work owners rather than one universal lifecycle rewrite.
3. **Remove demonstrated structural work:** F01/F02 are high-value preparation corrections; F03–F05 share relevant immutable/input preparation seams but retain their own closure obligations. F12's all-mode sparse layout and selected-method storage precede native admission changes. None requires a new generic cache or solver.
4. **Retain useful operation products:** F13 reuses neutral window workers; F14 retains exact point-dependent curvature before native weighting. O01's native checkpoint frontier and optional independent window waves need additional capability/demand/working-set decisions, not blanket eager retention or parallelism.
5. **Choose remaining numerical/protocol mechanisms from actual demand:** O02/O03 compare metadata grouping and derivative actions; O04/O05/O06 remain qualified capability alternatives. O07/O08 support feasibility and explanation without new mandatory managers, tracing payloads or benchmark gates.

A combined implementation must challenge interactions: broad type caches can stale mutable admission; eager derivatives/checkpoints can worsen the live set; a shared producer must not become one caller's child; response grants cannot shrink while SDK suffixes still own them; native worker retention must rebind cancellation/diagnostic scope; parallel windows must not multiply native teams/factors outside admission; and durable grouping must not create long solve transactions or mix IPC schemas. Removing original assessment or changing scientific tolerances is not an efficiency correction.

**Decision: Revise at source/interface evidence strength.** Adoptable corrections are bounded to the identified responsibilities, preserve the existing scientific authorities and library-owned numerics, and have revealing verification routes. A future plan should present the rule decisions and investigations explicitly, implement the chosen target directly and then exercise the formerly blocked public/scientific journeys. This review supplies no measured speedup, optimal solver guarantee, whole-product acceptance or completion claim for Plans39/40.
