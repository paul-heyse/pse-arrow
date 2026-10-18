---
title: Execution speed review audit and integrated numerical execution target
date: 2026-10-18
tier: design
purpose: target
decision: Revise
doc_role: review
doc_topics: [design, numerical-execution, performance, diagnostics]
standard:
  core: "3.4"
  efficient-architecture: "1.0"
  process-simulator: "1.5"
---

# Execution speed review audit and integrated numerical execution target

## 1. Assessment and boundary

**Decision: Revise.** The external [execution-speed review](../../pse-arrow-execution-speed-review.md) identifies substantial remaining work amplification. Its twelve directions generally survive source inspection, but several diagnoses, integration assumptions and prerequisites need correction. The strongest target is an extension of the existing mathematical, native, workflow and publication owners: propagate the consumer's actual numerical demand, retain compatible preparation and completed products, and schedule physical work according to actual dependencies. It does not require another solver framework, generic cache or orchestration platform.

The clearest corrections concern the source-edit route, existing explicit live-cursor detachment, Markowitz storage lifetime and the feasibility of symbolic reuse for its value-dependent pivots. Additional integration detail matters too: IDAS adjoint checkpointing is already used, PETSc ILU is an existing comparison, and compute lending is already coordinated. The pairwise symbolic Hessian-action diagnosis applies to flat arithmetic; guarded/provider programs use a different mixed-coordinate route. Live publication does use the append queue, but creates and flushes a fresh queue per sample. These distinctions change both the work to remove and the alternatives worth comparing.

Plan45's compatible-layout KLU symbolic retention and consumed physical-premise reuse are already implemented. AF07 concerns the separate response-factor constructor; it is not a claim that every native KLU application still repeats analysis. The study frontier and serial initialization also already retain useful owners.

The most consequential additional finding is a current effect bypass: same-demand assembly cache hits can skip required provider-backed selection/restriction visits at unchanged numerical inputs. The nested provider protects its own reuse correctly when called, but its enclosing cache can prevent that call. Fix this before broadening completed-product caching (AF19).

The audit supplies a position on **F01–F12 and the subsidiary recommendations in §§2–10**, nineteen detailed findings, an integrated target, conditional investigations, and rule impacts. Diagnosis evidence is **Implemented/source-inspected** unless stated otherwise; every corrective design is **Proposed**. No replacement or speedup was measured. A native control failed during compilation before any simulation executed; §9 records that boundary.

**Scope.** Design tier, target purpose. Baseline `5a009dccc7023e0142784892665c58bd20fbc305`, the same commit named by the external review, with its user-supplied untracked document preserved unchanged. The accountable reviewer is the Codex coordinator, assisted by separate mathematical, native-library and workflow inquiries and a fresh independent integration assessment. The [supporting evidence note](../evidence/execution-speed-review-audit-2026-10-18/inquiry-and-verification.md) records source routes, library versions and verification limits.

Judgment uses Core 3.4, Efficient Architecture Heuristics 1.0, Process Simulator 1.5 and the pse-arrow binding. The functional target is the profile's *Functional target* and blueprint §0/§0.2: interactive edit/re-solve without structural rebuild for value changes; studies, sensitivities, optimization and fitting; recycles/tears; dynamics from the same definitions; extension with units, properties and solvers; and independently compilable/testable responsibilities with their actual dependencies. Existing architecture, ADRs, policies and pins describe the current design. They are not reasons to reject a better target; §10 records consequential changes to them.

The review covers source admission, selected mathematical preparation, provider composition, sparse assembly, root/NLP execution, responses, fixed-mass ODE/index-1 dynamics, fitting/studies and result delivery. It does not newly certify thermodynamic models, all solver classes, higher-index/general implicit DAEs, sustained throughput, long horizons, GPU execution or deployment-wide memory completeness. Existing Plan45 completion is preserved; its historical candidate results are leads, not reopened tasks. Follow-up disposition belongs to a future authorized implementation plan. This review changes no product code, plan status, ADR or architecture owner.

## 2. Complete recommendation reconciliation

“Retain” preserves a direction, not a claim that its proposed implementation is already qualified. “Correct/improve” separates the valid cause from an unsuitable description or remedy. No entire numbered direction is omitted; specific claims and unconditional adoptions are omitted below.

| External item | Audit position | Required refinement and route |
| --- | --- | --- |
| F01 selected implicit derivatives | Retain and improve | Carry demand through provider composition and pre-effect admission, not only `ProviderRequest`; retain full physical inputs/state solve. AF01, AF13–AF14. |
| F02 compact sparse first products | Retain and improve | Preserve the review's qualification that regions already narrow active axes. Separate compact returned products from compact frame/region storage; coloring is conditional. AF02. |
| F03 contracted Hessian actions | Correct and improve | Pairwise symbolic construction is the flat route only. Compare direct contractions, retained first partials and sparse pairs; exact Hessian entries remain a distinct consumer demand. AF03. |
| F04 formulation/initialization | Retain and improve | Root primal reconstruction differs from NLP KKT reconstruction; qualify each transform, coefficient/domain premise and original bound. No blanket admission of NLP passes. AF04. |
| F05 preconditioning/linear tier | Correct and improve | Markowitz history is local peak scratch; its pivoting replaces both phases. Reuse apply scratch first; compare existing PETSc ILU/Schwarz before new adapters. AF05. |
| F06 dynamic actions/checkpoints | Retain and improve | Preserve its explicit Diffsol/IDAS distinction; name the already integrated IDAS adjoint calls. Add checkpoint-limit enforcement finding separately. AF06, AF15. |
| F07 scalable responses/rank | Retain and improve | Separate symbolic plan, fresh numeric factor and action-executor scratch. Strict opaque-fill refusal remains an independent eligibility question. Sparse QR cannot impersonate SVD. AF07, AF16. |
| F08 provider binding/fill | Retain and improve | Bind actual fresh provider and immutable specification lifetime; retain invocation effects and failure-safe publication. AF08, AF13. |
| F09 warm traversal/completed products | Retain with correctness prerequisite | Successful instantiation and numerical completion can be reused. Fix the existing enclosing cache's effect bypass before broader reuse; equal coordinates cannot discharge current provider effects. Preserve callback shield and independent final assessment. AF09, AF14, AF19. |
| F10 frame traffic/cache keys/SIMD | Retain conditionally | Compiler write/liveness facts can remove project-owned traffic. Symbolica's current public evaluator still copies into/out of its stack; a borrowed direct view is not established. Scalar keys concern preparation. AF10. |
| F11 completion-driven blocks | Retain and improve | Cover solve blocks and parallel initialization separately; change actual-overlap admission, finite aggregate work and failure semantics before removing wave barriers. AF11. |
| F12 live samples/result reads | Correct and improve | Integrator encoding and fresh per-sample queue flush are concrete costs. Explicit detachment already exists. Preserve original sample ordinals, durable ACK prefixes and bounded visibility; decoder lanes must actually be independent. AF12. |

The following local identifiers account for unnumbered recommendations; they are coverage references, not new finding or work-status identifiers. Repeated executive and sequencing proposals inherit the referenced position.

| Coverage reference / external location | Position and complete treatment |
| --- | --- |
| U01 §2 five lifetimes | Retain; add actual invocation/effect validity separately from numerical point equality (§3). A reusable worker is not permission to reuse a root, IVP or provider effect. |
| U02 §2 three objectives | Retain interactive latency, large-case latency and fixed-resource throughput as separate choices (§8). |
| U03 F01 full jets/dense implicit responses | Retain where fully consumed; preserve the review's explicit warning that sparse residual support does not imply sparse implicit responses (AF01). |
| U04 F02 coloring/library AD | Conditional comparison after exact support/decoder/provider capability qualification; no new bespoke AD subsystem selected (AF02, §7). |
| U05 F03 limited-memory Hessian | Retain its explicit distinction from exact entries; an HVP or limited-memory mode does not satisfy requested exact Hessian callbacks (AF03). |
| U06 F04 BTF/causal/coalesced/condensed forms | Retain comparisons, including fill/property cost and original reconstruction. No automatic scalar-root uniqueness or arbitrary SCC split (AF04–AF05). |
| U07 F04 starts/continuation/tangent predictors | Retain existing scientific selection and independent acceptance; factor/layout reuse is distinct from permission to transport a numerical start (§3, AF04). |
| U08 F05 native globalization/setup/Eisenstat–Walker/Anderson/FGMRES | Retain bounded profile comparisons. These capabilities largely exist; no custom Newton/globalization or universal default change (§7–§8). |
| U09 F05 threaded roots/SuperLU/PETSc processes | Conditional after same-result complete-route evidence. Vendored headers are not enabled linkage; use existing adapter/child owners (§7, RC07). |
| U10 F06 forward/adjoint/checkpoint tradeoffs | Retain; compare selected axes/functions and current fixed spacing first. Its native-IDAS lifecycle lead is sound; specifically improve the already integrated lifecycle (AF06, AF15). |
| U11 F07 sparse rank/regularity | Conditional separately named method/scaling/cutoff product; preserve dense SVD spectrum semantics and independent action availability (AF07, RC06). |
| U12 §5 study concurrency | Retain the review's preservation of current `FuturesUnordered` completion scheduling, bounded preparation and continuation ownership (§4). |
| U13 §5 fitting load balance | Retain, first compare stable affinity-aware assignment. Experiment stride and dynamic profile-chain dispatch are different paths (AF18). |
| U14 §5 total teams/process gates | Retain its warning about oversubscription; add the actual existing coordination. Active phases, resident state and process exclusion differ (AF11, §4). |
| U15 §6 unread subscribers/detachment | Preserve explicit close/drop detachment and bounded backpressure. Automatic loss/detach or early ACK changes meaning; no implicit adoption (AF12, RC09). |
| U16 §6 parallel result reads | Conditional bounded file-affine decoder lanes/prefetch. Cloning the current mutexed decoder does not create parallel decode (AF12). |
| U17 §6 source persistence | Correct the production route: `SourceDelta` and changed-row builders exist. Narrow full parent/index/reference/serialization work in the actual delta owner (AF17). |
| U18 §6 cache capacity/protection granularity | Conditional physical-product layout inquiry. Historical grant/cache mismatch establishes neither current latency nor authority to lower charges or enlarge defaults (§7). |
| U19 §7 library versions/eligibility | Retain exact-version qualification. SDK version is not native version; current documentation is discovery. Full technical library eligibility remains; licensing/current consumer absence is not a veto (§7). |
| U20 §7 release/dist/PGO | Correct: release dependencies already use optimization level 3; dist already has FatLTO/one codegen unit. Compare actual hot artifacts; Rust PGO does not cover all native/generated code (§7). |
| U21 §7 JIT/subprograms/SIMD | Conditional changed-premise experiments, preserving Plan45's specific failed Second-order/stop-hook/ISA results. AD vectorization is not hardware SIMD (AF03, AF10, §7). |
| U22 §7 GPU/ISA/precision | Exploratory, requiring resident/batch amortization and explicit arithmetic/portability contracts. No GPU or relaxed-precision adoption from source availability (§7, RC11). |
| U23 §8 A–E order / S01–S09 | Improve prerequisites and add revealing cases (§5, §8); retain coverage, not an authorized implementation schedule. |
| U24 §9 diagnostics/measurement | Retain existing phase/work counters as a decision-driven menu. Separate waits, inclusive clocks, reservations, actual memory and successful/refused populations (§8–§9). |
| U25 §9 numerical tolerances | Retain distinct work, physical, integration, derivative, rank and KKT requirements. Ordinary engineering accuracy stays 0.001; no tightening or weakening selected here. |
| U26 §10 foundations and seven impact categories | Independently reassessed in §6 and expanded to exact RC rows in §10; neither external verdicts nor pending extensions are implementation acceptance. |

## 3. Integrated target: responsibility, validity and storage

The existing decomposition is suitable. Authored relations and physical admission own scientific meaning; mathematical preparation owns conservative support, guards, differentiation and layouts; providers own current property/implicit behavior; native libraries own numerical iteration, globalization and factors; workflows own scientific composition and scheduling; publication owns durable/result effects. Optimization should improve composition at these seams rather than teach consumers to reinterpret native or physical internals.

| Owner / hidden decision | Consumed and exposed contract | State/effect lifetime and local test boundary |
| --- | --- | --- |
| Modeling/compiler | Typed source, lexical binding dependencies, admitted physical definitions → selected model/structure | Immutable revisions and consumed dependencies; local source fixtures, no solver/store for pure binding tests. |
| Mathematical preparation | Complete selected obligations, support, actual derivative demand → stage/product layouts | Reusable structural preparation; fresh numerical inputs and separate current-effect checks. Pure/library evaluator fixtures. |
| Attempt provider/worker | Actual provider instance, immutable bound descriptor, current invocation context → completed demanded product | Private admitted scratch; numerical product and effect validity are distinct. Counting/failing provider fixtures need no canonical service. |
| Native adapter | Qualified class, original/reduced oracle, numerical policy → truthful report and independently assessed candidate | Native session/thread owns factors and callbacks. Structure can survive exact compatibility; numeric factors require actual matrix identity. Native-local tests. |
| Workflow | Scientific dependencies, starts, finite aggregate work, actual live resource envelope → committed candidate dependencies | Completion may unlock successors only after dependency commit; actual issued work drains. Scheduler tests can use controlled native surrogates. |
| Publication/read | Physical sample membership, selection/attempt identities and fence → provisional events, acknowledged prefixes and frozen reads | Bounded buffers/decoder lanes; only actual durable ACK advances coverage. Store-specific tests remain at this boundary. |

Five data lifetimes remain useful: structure; prepared numerical layout; attempt worker; current numerical point/product; result publication. **Invocation validity is a separate dimension.** Equal coordinates can justify arithmetic reuse while requiring another envelope, anchor, restriction, cancellation or effect visit. A completed-product key must state which immutable premises it discharges and which current checks remain. A provider without purity/retained-premise capability must not acquire one through an enclosing cache.

The numerical-demand contract distinguishes selected partial columns, JVPs, VJPs, weighted second actions, sparse Hessian entries and full jets. Physical inputs and original obligations remain complete. Composition must consume the requested action directly where supported: extending a request record while `ProviderLift` expands all local partials would leave the central inefficiency intact. Admission must use that representation before allocation, including factor/RHS/frame extents.

Immutable support and scatter maps may be shared; workers, numeric factors, action scratch and incomplete fills may not escape their declared owners. An `Arc`-shared immutable response factor cannot absorb unsynchronized mutable RHS scratch. One sufficient completed staging boundary can replace duplicate copies, but a foreign callback must still receive no partial write after late failure. Cache/reset/refusal transitions must preserve independently completed weaker products where currently allowed and invalidate unknown failures conservatively.

### Physical and numerical preservation

The scoped review does not change physical conventions. Their authority remains blueprint §§8–9 and the admitted model/provider operations, not a performance profile.

| Element | Dimension/unit and basis | Reference/convention | Envelope/authority consumed by execution |
| --- | --- | --- | --- |
| State, input, property and residual | Declared physical quantities and row units; material basis explicit | Admitted absolute/difference, component/phase and reference-state meaning | Model/provider admission and original equations; no solver-local reinterpretation. |
| Derivative/action | Output units divided by each physical differentiated input unit; directions/cotangents carry declared coordinates | Native normalization converts back to physical coordinates | Declared source/order, guard smoothness, branch neighborhood and linear-solve residual checks. |
| Dynamic rate/Newton action | State/time; residual action follows fixed-mass convention | ODE/index-1 `diag(I,0)` and actual algebraic mask; IDAS operator `cj M − J` | Original dynamic contract, current mode/shift, consistent initialization, event/schedule boundaries. |
| Fit contribution | Prediction and measurement share admitted units/basis; difference standard deviation | Existing declared loss and authored reduction order | Inclusion/importance, parameter scaling, derivative availability and independent final assessment. |
| Result sample | Original physical units and identities | Sample/attempt ordinal and selected accuracy-attempt identity | Provisional data, durable prefix and scientific acceptance remain distinct. |

**Well-posedness.** Fixed/free/parameter roles and equation support remain model-owned. Original incidence/matching/BTF checks reject structural defects with model identities before eligible native execution. A stream graph is not a dependency graph; matching is not numerical regularity. Root presolve must reconstruct every eliminated physical coordinate and retain original rows/bounds. Optimization additionally needs valid multiplier/KKT recovery. A scalar block is not automatically unique or bracketed; implicit selection and regular neighborhoods remain explicit. Dynamics keeps its fixed-mass ODE/index-1 limit unless independently extended.

| Numerical stage | Formulation / derivative source | Scaling / class and capability | Outcome, tolerances and original check |
| --- | --- | --- | --- |
| Preparation/provider lift | Original guards/branches and declared smoothness; library symbolic/Taylor products or qualified provider actions | Conservative support and actual selected physical axes | Unsupported demand refuses before work; no silent finite differences or zeros. |
| Implicit solve/response | Selected complete residual relation; IFT factor and forward/transpose RHS actions | Physical unknown/input coordinates with explicit scales; regular square system | Original residual and inactive-bound/selection checks; derivative backward-error tolerance remains separate from engineering accuracy. |
| Root/block execution | Consumer-qualified primal transformations, original reconstruction; analytic Jacobian/JVP | KINSOL/PETSc/library root ownership; exact actual preconditioned operator | Native status is not acceptance; fresh original rows/bounds and full assembled assessment remain. |
| NLP/fitting | Exact entries/actions or explicit approximation mode; no interchangeable curvature claims | Native NLP, declared loss, model scales, valid dual recovery | Requested feasibility/stationarity/KKT/derivative criteria retained independently. |
| Dynamics/adjoint | Current modes/schedules/events; direct or assembled actions under same operator | Native BDF/other admitted methods, actual mask and shift; selected sensitivity axes | Integration/physical/error checks, replay work and checkpoint ownership; no changed-parameter history reuse. |
| Response/rank | Fresh original Jacobian and checked factors | Separate sparse actions, dense SVD spectrum or explicitly named alternative | Factor solve/backward error, QR pivot rank and singular spectrum are different claims. |

## 4. Findings and corrective contracts

Each AF identifier is local to this audit. The correction and benefit remain **Proposed**; source evidence establishes the cause, not a measured speedup. Verification describes what a future implementation must distinguish. AP-07/G9 and PS-11 support unnecessary-work findings; more specific principles/scenarios are named where material.

<a id="af01"></a>

### AF01 — Implicit derivative execution expands beyond the demanded product

**Cause/evidence:** `pse-math/implicit.rs::derivatives_at_verified_point` forms all residual coordinates, `n × p` first RHS, every parameter-pair second RHS and `n × p × p` Hessians; `Nested::project` selects afterward. `ProviderRequest` has outputs/order but no selected input/action demand. `jets.rs::ProviderLift` composes generic local partials. The current construction preflight also assumes complete derivative extents.

**Consequence:** S01/S02 narrow sensitivity/action consumers pay for complete parameter jets and can be refused by complete-jet limits before their much smaller requested operation runs. Sparse residual support does not resolve this. **Correction:** provider/math owners define and compose selected columns and directional/transpose/weighted products, with representation-specific admission and complete physical inputs/state solve. Keep full jets for full consumers. Faer forward/transpose multi-RHS solves are viable components, not a complete second-order integration. **Preserve/verify:** selection, original residual, unknown-bound neighborhoods, effect visits and derivative residual checks. Vary total parameters independently from demanded directions and count actual derivative components/RHS/storage; test full demand, order upgrades, refusal and recovery. AP-02/AP-07, PS-07/PS-11, G6/G9.

<a id="af02"></a>

### AF02 — Sparse assembly receives dense local first-result and frame storage

**Cause/evidence:** `execution.rs::evaluate_seeded` sizes a whole-worker slot-by-width frame and selected outputs' full first rows; `assembly.rs` later scatters structural support. Preserve the external review's qualification that arithmetic regions already narrow active axes. **Consequence:** S02 large sparse flat bodies retain/move zeros despite small final support. **Correction:** separately introduce support-indexed first results and compiler-owned compact stage/frame layouts; compare selected library partials or support-shaped Taylor products before coloring. Full/dense representations remain valid where cheaper. **Preserve/verify:** conservative branch/provider support, structural absence versus numerical zero, contribution order and scale. Compare flat versus many-instance equivalents with independently varied support density; count frame, region, result and arithmetic work separately. AP-05/AP-07, PS-07, G9.

<a id="af03"></a>

### AF03 — Flat Hessian-action preparation enumerates second pairs before contraction

**Cause/evidence:** `products.rs::compile_direct_product` is selected for flat expressions, unions second-support pairs, and repeatedly differentiates the weighted scalar for each pair. Guarded/provider programs use the existing mixed-coordinate Taylor route instead. **Consequence:** S02 dense curvature can make action preparation proportional to a Hessian that no consumer requested. **Correction:** compare directional differentiation of the weighted gradient, gradient of the weighted directional expression, retained first partials within the existing pair route, and sparse pairs. Preserve exact-entry programs for Ipopt and separate limited-memory policy. **Preserve/verify:** the original value-stage guard/smoothness pass survives zero weights or symbolic cancellation. Dense reductions, sparse chains and repeated subexpressions must compare construction, instruction/storage size and repeated action work. Current Symbolica hyperdual machinery is interface evidence, not a qualified reverse-mode HVP replacement. AP-03/AP-07, PS-06/PS-07/PS-09, G6/G8/G9.

<a id="af04"></a>

### AF04 — Root transformation eligibility inherits an unnecessarily broader recovery question

**Cause/evidence:** `execution/runner.rs::roots` normalizes/checks/executes/reconstructs without the NLP presolve pipeline. `presolve.rs` withholds bound contraction because original-bound dual recovery is unavailable. **Consequence:** S03 roots cannot use potentially useful exact primal-only reductions merely by sharing optimizer eligibility. **Correction:** one transformation owner exposes consumer-specific obligations: exact primal reconstruction for roots; additionally multiplier/KKT recovery for optimization. Start with alias/affine comparisons, separately qualify interval contraction, BTF/coalescing and implicit condensation. A tolerance-small coefficient is not automatically negligible. **Preserve/verify:** guards, all eliminated bounds/rows, physical scales and final original assessment; library reuse fingerprints also need caller expression/provider revision identity. Probe unequal scales, tiny coefficients, changed providers and reconstruction failures. AP-02/AP-04/AP-07, PS-04/PS-07/PS-10, S03, G6/G9.

<a id="af05"></a>

### AF05 — Block preconditioning repeats apply allocation and has a costly factorization admission route

**Cause/evidence:** `kinsol/block.rs::Blocks` uses diagonal BTF block Jacobi, fresh FERAL Markowitz per setup and fresh global/per-block RHS vectors per apply. Its conservative reservation includes `2048 n² + 256 n³ + 4096 n`. Pinned Markowitz source makes pivoting value-dependent and returns factors without local row-membership/lazy-heap scratch. **Consequence:** S04 repeated applications allocate, while a large SCC can be refused by a peak reservation retained in session accounting. The cubic allowance is not measured actual factor memory. **Correction:** retain apply scratch/maps now; compare current KLU/BlockFactor and existing PETSc GMRES/ILU or nonlinear Schwarz. Reusable symbolic alternatives require a different qualified factor path (explicit Gilbert–Peierls, KLU or faer), not extracting a symbolic phase from Markowitz. Stronger triangular/overlap/Schur preconditioning needs actual operator, split, refresh and failure contracts. **Preserve/verify:** true coupled JVP, all-factor invalidation on partial setup failure, solves rather than explicit inverses. Compare weak/strong coupling, dominant SCCs, nonnormal matrices, fill and total accepted/refused operations. Threaded root selection remains conditional. AP-03/AP-05/AP-07, PS-09, G8/G9.

<a id="af06"></a>

### AF06 — Dynamic action consumers assemble intermediates and lack a coupled DAE preconditioner

**Cause/evidence:** Diffsol `dynamics/integrator.rs::{product,transpose_product}` obtains assembled raw partials then multiplies; some scatter coordinates are searched repeatedly. IDAS `jtimes` already executes a direct action; its Krylov route allows None/Jacobi and rejects BlockFactor. **Consequence:** S05 action-heavy trajectories can assemble unconsumed derivatives; difficult algebraic coupling may lack a suitable preconditioner. **Correction:** adapt the existing in-place Diffsol action traits with prepared maps; retain assembled reuse for direct/factor-heavy consumers. Compare DAE preconditioning of the actual `cj M − J`, including algebraic rows, shift/mode changes and consistent initialization. **Preserve/verify:** forward versus transpose signs, selected parameter axes, event/schedule semantics and existing error controls. Compare complete action-heavy and factor-heavy trajectories, not isolated multiplies. Native IDAS adjoints are already integrated; improve them under AF15. AP-02/AP-03/AP-07, PS-07/PS-09, G6/G9.

<a id="af07"></a>

### AF07 — Response construction repeats compatible symbolic work and action scratch

**Cause/evidence:** `square_response/actions.rs::SparseFactor::prepare` performs symbolic and numeric faer LU per construction; action groups allocate RHS, solution, residual and MemStack buffers. **Consequence:** S06 repeated same-layout responses redo structural work and repeated actions allocate. **Correction:** share exactly compatible immutable symbolic plans, use fresh matrix-specific numeric factors, and put mutable scratch with each action executor, not the escaping shared factor. **Preserve/verify:** point/scaling/support/ordering identity, completed publication and concurrent action isolation; count symbolic versus numeric work independently, singular→regular transitions and late failures. A separately named SPQR/iterative regularity product is conditional; preserve the public scaled full-SVD spectrum and cutoff. Strict storage eligibility is AF16, not solved by symbolic reuse. AP-02/AP-07, PS-07/PS-10, G6/G9.

<a id="af08"></a>

### AF08 — Warm provider dispatch repeats immutable binding and returns temporary owned jets

**Cause/evidence:** `execution.rs` hashes `spec.key`, map-lookups/checks binding and validates request/result each call; `pse-kernels` constructs ordinal uniqueness sets. Nested projection allocates owned selected arrays. **Consequence:** S01/S02 frequent small callbacks pay descriptor and temporary-allocation costs unrelated to fresh physics. **Correction:** precompute keys, bind the actual fresh attempt provider slot and enforce immutable descriptor/request extent for that binding; use admitted caller fill buffers or an internal expiring borrowed view. Retain checks at uncontrolled public boundaries. **Preserve/verify:** per-invocation envelope/finiteness, scope/deadline, selection/effect and capability restrictions. Partial fill never becomes complete; test mutable descriptor attempts and late failures. Symbolica's infallible external-function callback is not a substitute for the fallible provider contract. AP-01/AP-02/AP-07, G3/G4/G5/G9.

<a id="af09"></a>

### AF09 — Warm traversal and callback assembly repeat already completed work

**Cause/evidence:** `Worker::prepare_order` traverses stage instantiation even when prepared (twice on a cold construction). CaseWorker gathers/bits-compares local inputs and refills assembled products on repeated demands; native/oracle layers repeat full-point finite scans and completed-buffer copies. **Consequence:** S01/S02 repeated callbacks incur stage/coordinate/assembly traffic despite completed compatible products. **Correction:** successful per-order instantiation state; exact owner-issued point/fixed-value/demand/weight/direction revisions; one sufficient failure-safe callback staging owner. Keep useful local-group reuse when only part of a global point changes. **Preserve/verify:** provider effects cannot be skipped solely by point identity; reset/incomplete instantiation and unknown provider failures invalidate appropriate state. Preserve independently complete weaker products, accumulation order, no partial foreign output, unwind containment and fresh final original assessment. Real callback traces must include equal free values with changed fixed inputs, dynamic restrictions, order upgrades, reset and late failure. AP-02/AP-07, PS-10/PS-11, G4/G5/G6/G9.

<a id="af10"></a>

### AF10 — Frames and warm scalar lookup repeat avoidable project-owned traffic

**Cause/evidence:** evaluator frame poisoning/zeroing/gather/scatter protect unassigned/nonfinite states; `library/admission.rs` clones scalar key vectors before lookup and performs multiple map operations. **Consequence:** S02/S09 small arithmetic can be dominated by movement/preparation metadata. **Correction:** compiler write/liveness facts and contiguous owned views remove proven redundant traffic; borrowed scalar lookup or an exact prepared handle retains one cache authority. **Preserve/verify:** writes before reads across branches, structural zeros, failure/reset validity and original obligations. Current Symbolica API copies its private stack; no borrowed stack API was established. SIMD/batch realization is a conditional pure-homogeneous comparison, preserving divergence/source failure/cancellation/arithmetic requirements. No broad removal of poisoning from intuition. AP-05/AP-07, G3/G6/G9.

<a id="af11"></a>

### AF11 — Wave barriers and repeated ownership delay dependency-ready blocks

**Cause/evidence:** `initialization::ready_wave` rebuilds producer maps/scans dependencies. Solve blocks clone complete values, create fresh parallel `Retained` owners, join the whole wave, then commit. Parallel initialization instead prepares waves and `join_all`s existing session tasks; serial initialization already retains one session and per-block bindings. `prepare_blocks` computes the resident/team envelope over simulated canonical waves. **Consequence:** S07 a fast predecessor's chain waits for an unrelated slow sibling; repeated maps/copies/owners add work. **Correction:** prepared actual execution adjacency, compact consumed bindings and existing bounded MathService lanes; unlock successors only after successful prerequisite commit. Retain backend-qualified neutral scratch separately from numerical starts/history. **Prerequisites/preservation:** actual-overlap admission must replace old wave maxima; parent compute must be lent before child acquisition. Reserve finite aggregate iteration permission before dispatch and reconcile actual work at completion; subtracting after completion alone can oversubscribe the same remainder. Account for all issued failed work, stop new dispatch after known terminal failure, drain on cancellation and choose visible failures deterministically. A successor executing before a later unrelated failure changes physical work; provisional dependency commits remain distinct from accepted atomic stage completion, and no rollback can erase issued provider/native effects. Verify skewed sibling/chain graphs with mixed team/memory widths, failure races and exhausted budgets. AP-03/AP-05/AP-07, PS-05/PS-08/PS-09, G5/G9.

<a id="af12"></a>

### AF12 — Live sample and analytical read lifetimes serialize otherwise separable work

**Cause/evidence:** `live_simulation::Observer::sample` constructs rows/checked Arrow in the native callback; full-channel retry polls. `run.rs` serially awaits subscriber, append and recorded delivery. `append_live_sample_batch` constructs/flushes a fresh queue per sample. Result scans advertise one partition, and one shared decoder mutex spans file open/decode. **Consequence:** S08 dense output pauses integration and repeats publication setup; independent files cannot decode concurrently. **Correction:** bounded compact physical capture, off-callback checked encoding and larger physical chunks/retained live queue at the existing durable owner. Preserve individual public sample ordinals and explicit close/drop detachment. For reads compare bounded file-affine decoder lanes/prefetch, with reservation at actual decode admission rather than several queued reservations behind one mutex. **Preserve/verify:** provisional versus recorded/selected attempts, exact durable prefix ACK, latency/flush barriers, bounded unread-subscriber backpressure, terminal close/cancel/drain, frozen read tickets and integrity/schema/row checks. No early ACK or implicit lossy detach. Compare output densities and closed/unread consumers, then real independent multi-file decoders. AP-01/AP-05/AP-07, PS-10, G2/G5/G9.

<a id="af13"></a>

### AF13 — Inner implicit trials and projections allocate or clone reusable result storage

**Additional detail beyond the external prescriptions. Cause/evidence:** `implicit.rs::evaluate_trial_with` calls allocating `Worker::evaluate_trial` despite `evaluate_trial_into`; general nested reuse clones the cached point and full `ProviderValues` before projection, while the fixed fast path borrows. `check_solve` allocates row sums/scales per verification. **Consequence:** S01 repeated inner residual calls and cache hits still allocate/copy complete products. **Correction:** admitted residual-worker-owned result and verification scratch; borrow completed products after required configuration/selection effects or fill the requested result. **Preserve/verify:** locks/borrow lifetime, current effects, cancellation, complete-publication and CSC verification accumulation/absolute scaling. Count allocations/cloned entries for residual, cached projection and verification independently; late failure then success must not expose stale data. AP-07, PS-07/PS-11, G5/G9.

<a id="af14"></a>

### AF14 — Implicit derivative-order upgrades discard compatible factor and first products

**Additional opportunity. Cause/evidence:** First→Second demand calls `derivatives_at_verified_point` again, refactors `r_z` and recomputes first RHS before second work. The same factor workspace also serves affine inner solves. **Consequence:** S01 order upgrades repeat already completed numeric work. **Correction:** associate verified factor/first products with exact point, physical inputs, actual matrix values, selection/provider/configuration and completion; retain missing richer residual derivatives separately. Presence of a numeric factor is not that association. **Preserve/verify:** effectful calls can invalidate matrix identity even at equal coordinates; failures clear affected validity. Count first solves/factors on an upgrade, changed fixed data, affine solve workspace reuse, selection change and recovery. AP-02/AP-07, G4/G6/G9.

<a id="af15"></a>

### AF15 — IDAS checkpoint refusal occurs after a multi-checkpoint native call

**Additional finding. Cause/evidence:** `dynamics.rs::AdjointSettings` declares at most `max_checkpoints` held at once. `idas.rs::Session::solve` calls `IDASolveF(..., IDA_NORMAL)` and checks `held > limit` only after return. Pinned SUNDIALS `idas/idaa.c::IDASolveF` loops internal steps, allocates/appends checkpoints, and returns the count afterward. The configured checkpoint limit is therefore a post-call refusal, not enforcement before each possible checkpoint allocation. **Consequence:** S05 a long target interval can exceed the configured checkpoint population inside one call before the Rust owner refuses it. This is not a measured OOM or proof that all other resource bounds fail. **Correction:** use the native single-step/checkpoint lifecycle or an equally sound native pre-allocation control to refuse before exceeding admitted population, preserving interpolation/events/schedules and actual accepted-step/replay accounting. Do not implement a new integrator. **Verify:** one long target versus shorter targets, small checkpoint limit, exact boundary, failure/cancellation and cleanup; observe population before return, not only terminal refusal. Existing max-step caps are different limits. AP-05/AP-07, DP-20, S05, G7/G9; no silent scientific truncation was found.

<a id="af16"></a>

### AF16 — Faster sparse responses would still refuse strict opaque-fill admission

**Additional target gap, not a general correctness defect. Cause/evidence:** `square_response/actions.rs::admit_storage` declares unknown private faer symbolic/numeric heap fill; runtime `strategy/admission.rs` correctly refuses opaque strict linear storage before evaluation. The deployment allowance is not a complete producer bound. **Consequence:** S06 symbolic reuse alone cannot make this route eligible when strict storage is requested. **Correction:** explicitly decide whether the required consumer needs strict eligibility; if so compare producer-owned allocation knowledge, an alternative factor owner or a deliberately changed policy. Preserve ordinary deployment-local execution and conservative unknown refusal meanwhile. **Verify:** actual route/profile admission before derivative effects, retained-factor action storage and failure recovery. Do not infer a bound from RSS or weaken the rule indirectly through a reuse cache. AP-02/AP-05, G7; source-established limit, remedy unresolved pending consumer/policy choice.

<a id="af17"></a>

### AF17 — Source deltas still rebuild broad parent/reference/canonical products

**Corrected external diagnosis. Cause/evidence:** ordinary `with_declarations` already forms `SourceDelta`, and changed-row builders are narrow. `canonical/delta.rs` reconstructs the parent inventory/index, re-resolves every outgoing reference and serializes every merged declaration before unchanged-object filtering. The old `persist(previous)` branch is not the production edit route. **Consequence:** S01/S09 a small source edit does work proportional to the parent model. **Correction:** immutable typed parent/index/canonical products plus actual binding-dependency invalidation; serialize affected payloads and retain complete source identity/reconciliation. **Preserve/verify:** absent/shadowed/imported bindings, names, deletions, data/physical visibility, ordered declarations and document spans. Full ingress may legitimately remain complete. Count affected versus all-reference work with shadowing/deletion/physical and equal-identity controls; no raw changed-ID shortcut. AP-04/AP-07, PS-11, G1/G6/G9.

<a id="af18"></a>

### AF18 — Fitting experiments use static stride despite heterogeneous repeated cost

**Retained unnumbered opportunity. Cause/evidence:** `fitting/experiments.rs` assigns retained lanes by `(lane..count).step_by(lanes)`, each with full-length optional worker inventory. Profile-chain dispatch separately uses dynamic task claiming; it is not the same defect. **Consequence:** S07 uneven experiments can strand compute and amplify empty lane metadata. **Correction:** compare stable cost-aware/affinity-preserving assignment and compact lane inventories first, then dispatch genuinely unstarted work if it wins without state replication. **Preserve/verify:** one private experiment owner, no unapproved replay after failure, finite shared team/storage, sequential scientific pins and authored-order atomic reduction. Repeated skewed experiments must count worker reconstruction as well as idle/total time. AP-03/AP-07, PS-09/PS-11, G5/G9; load-dependent runtime importance remains unmeasured.

<a id="af19"></a>

### AF19 — Enclosing same-demand caches bypass required current provider effects

**Additional correctness finding. Cause/evidence:** `CaseAssembly::worker_scoped` creates group caches for provider-bearing programs without an effect qualification. `CaseWorker::evaluate_inner` hits on input bits/order; `evaluate_bound_product_inner` additionally keys weights/direction. Both return without invoking the worker/provider. `assembly_projection.rs` excludes provider stages only from cross-demand projection, not these direct hits. Compiler implicit admission permits registered providers in selection anchors/restrictions, and runtime `MathService::worker` constructs their attempt providers without excluding them from CaseWorker.

`SelectionWorker` explicitly requires current provider-backed effects even at equal inputs. `Nested::evaluate_inner` runs effectful anchors before its numerical key and restrictions on reusable jets, but only when called. **Consequence:** S01 successfully evaluate a provider-bearing constraint/action, leave all cache-key values unchanged, change the operational provider's current anchor/refusal, then request it again: the enclosing hit can return the old result without the required current visit. A changed anchor can alter root selection; a current restriction can refuse a formerly usable product. This is source-established reachability, not an executed numerical counterexample.

**Correction:** simplest safe behavior disables direct hits for provider-bearing programs and invokes the provider owner, which can retain its own qualified numerical data while executing required effects. A more selective completion-refresh/effect capability is optional further design; no global purity inference from bits or descriptor identity. Apply the same rule to selected partials and weighted/actions. **Preserve/verify:** independently completed numerical data may remain stored after a typed refusal, but its existence does not authorize a new invocation. Add an enclosing CaseWorker control with unchanged keys and changed effect/refusal, including order upgrade, cross-demand projection, weighted/action hits, cancellation and pure arithmetic reuse. Existing direct-provider operational-selector tests do not exercise the outer cache; assembly tests cover changed keys and retained lower completion but not this unchanged-key effect change. AP-02/AP-04/AP-05, G4/G6/G9 and potentially PS-G3; no claim that every final scientific result is wrong.

## 5. Revealing change and workload scenarios

These scenarios refine the external S01–S09 rather than prescribe a benchmark campaign.

| ID | Stimulus / kind of change | Expected boundary and acceptance |
| --- | --- | --- |
| S01 | Value edit, same-point order upgrade, changed operational provider effect, or new provider model using existing physical concepts | Instance/binding extension stays with source/provider owners; structure survives value overlays, demand/effects remain explicit, fresh science remains checked. AF01/AF08/AF09/AF13/AF14/AF17/AF19. |
| S02 | Flat sparse body, dense reduction and equivalent many-instance composition | Representation/mechanism choice depends on actual support/demand; compact results are distinguished from compact arithmetic/frame work. AF02/AF03/AF10. |
| S03 | Alias/affine causal chain versus simultaneous root/NLP, with a recycle or ambiguous scalar root | Transformation/strategy variation preserves original reconstruction, bound/selection meaning and separate KKT obligations. AF04. |
| S04 | Weak/strong BTF coupling, dominant SCC, pivot/fill growth, poor start | Solver/preconditioner replacement remains adapter-owned, keeps the true operator and same accepted result; no assumed threaded/Schur winner. AF05. |
| S05 | Stiff index-1 trajectory, shift/mode/schedule changes, selected forward/adjoint demand and checkpoint pressure | Same scientific definitions; direct/assembled routes and checkpoint enforcement preserve native iteration, consistent starts and all error criteria. AF06/AF15. |
| S06 | Repeated response point/layout, near rank loss, transpose/batched RHS and strict storage request | Mechanism/new product distinction: actions, factor eligibility and numerical-rank claims stay independent. AF07/AF16. |
| S07 | Skewed sibling plus successor chain, unequal fitting experiments, mixed teams, cancellation and failure | Workflow composition unlocks actual dependencies within admitted overlap/work; all issued work drains and deterministic semantic outcomes remain. AF11/AF18. |
| S08 | Dense live output, slow/unread/closed subscriber, uncertain append, multi-file frozen read | Boundary representation change preserves original sample membership, exact ACK coverage, integrity and bounded queues/decoder state. AF12. |
| S09 | Local source edit, independent policy/provider test, optional cache pressure or evaluator replacement | Actual dependency-local development; precise lexical invalidation and one cache authority; no store/native startup for pure responsibilities. AF10/AF17 and §7. |

## 6. Foundation and gate judgment

The judgments apply to the inspected execution boundaries, not every unit/property model. Proposed extensions are not existing guarantees; where their settling evidence is missing they remain explicitly unaccepted below.

| Foundation | Verdict | Evidence and consequence |
| --- | --- | --- |
| AP-01 separation | Satisfied in inspected ownership | Mathematical/provider, native, workflow and publication responsibilities are separable; retaining those owners avoids a new cross-cutting framework. |
| AP-02 contracts | Violated at provider reuse; other extensions unresolved | Equal-input cache permission crosses the required current-effect contract (AF19). Selected actions, completion commits and alternative rank additionally need explicit contracts before adoption. |
| AP-03 composition | Satisfied with targeted changes | Native libraries own algorithms; existing action traits, MathService lanes and append owner can compose improvements without duplicating them. |
| AP-04 domain/authority | Violated at enclosing provider cache | The nested owner models current selection/restriction effects, but outer same-demand reuse bypasses that semantic authority (AF19). Other inspected physical/class/reconstruction owners remain strengths. |
| AP-05 structure/constraints | Violated in effect/checkpoint enforcement; new scheduling unresolved | AF19 omits an effect premise from reuse permission; AF15 checks checkpoint count after native allocation opportunities. AF11's old wave envelope cannot qualify a new overlap schedule. |
| AP-06 local reasoning | Satisfied in inspected responsibilities | Pure math/provider/policy tests have local inputs; store effects and native adapters retain their actual dependencies. Broad development-build qualification was not repeated. |
| AP-07 execution fit | Violated | S01/S02 full implicit jets and dense intermediate products and S07 wave barriers/repeated owners amplify supported operations without a consumed requirement. S08 repeated encoding/setup is demonstrated; cross-sample batching additionally depends on the visibility guarantee consumed. |

| Gate | Judgment / evidence boundary |
| --- | --- |
| G1 authority | Pass, source-scoped: named physical, derivative, source and publication owners remain. No second cache/semantic definition is recommended. |
| G2 fidelity | Fail at AF19: current selection/restriction invocation meaning can be silently dropped on an enclosing cache hit. Other retained meanings remain scoped strengths; proposed rank/output/transform changes need explicit contracts. |
| G3 validity | Fail for current restriction bypass (AF19): a previously completed product can be reused without the current required validity visit. Other inspected rejection/staging strengths do not offset it. |
| G4 hidden behavior | Fail at enclosing same-point provider reuse (AF19): required current effects can disappear on a cache hit. Proposed binding/cache remedies cannot assume purity. |
| G5 consistency/recovery | Pass for inspected completed staging and durable-prefix paths; completion scheduling/batching alternatives remain unqualified until failure/cancel cases pass. |
| G6 transformation/reuse | Fail at the provider-effect boundary (AF19); otherwise original reconstruction, reset and current-point checks are preservation constraints, not permission to bypass them. |
| G7 truthful capability | Fail for treating configured IDAS checkpoint count as pre-allocation enforcement (AF15). Sparse response strict refusal is truthful, not failure. This review makes no speedup or candidate capability claim beyond inspected evidence. |
| G8 library leverage | Pass for current algorithm ownership; candidate direct products/stronger factors/QR/threaded paths remain interface-qualified or proposed, not accepted integrations. |
| G9 architectural fitness | Fail through AP-05/AP-07 and the concrete scenarios above. Correct current outputs do not settle execution fit. |
| PS-G1 physical consistency | Pass for inspected contract transport/check owners; actual thermodynamic/closure breadth not requalified. |
| PS-G2 well-posedness | Pass for inspected original structure/matching/class rejection; no numerical regularity inferred from BTF. |
| PS-G3 numerical integrity | Unresolved for the final-result consequence of AF19: invocation semantics are demonstrably bypassed, but no numerical/final-publication counterexample was executed. Existing status/original-check safeguards remain required. |

Behavioral adequacy fails the provider invocation/reuse contract and remains unresolved for its final numerical-result consequence; architectural fitness fails independently. The overall **Revise** decision follows known AP-02/AP-04/AP-05/AP-07 gaps, without claiming that every supported simulation is incorrect or that unresolved remedies are implemented.

## 7. Library fit, alternatives and conditional investigations

Exact local sources and feature ownership take precedence over current documentation when describing implementation. The library skills and Context7 were used for discovery; inspected baseline versions are Symbolica 3.0.1, faer 0.24.4, Diffsol 0.16.2, SUNDIALS native 7.1.1 through `sundials-sys` 0.6.2, PETSc 3.24.0 and the POUNCE/FERAL fork revision in Cargo.toml. The math skill's Symbolica 3.0.0 evidence cannot by itself qualify 3.0.1. Upstream latest/main examples cannot silently replace the pinned APIs.

Direct actions and completed matrices must have one coherent reuse relationship at existing assembly/product owners: consume an eligible current matrix when it is already required, otherwise prepare the demanded action. Adding both as unconditional parallel preparations could repeat work and retain two products at the same point. Existing cross-demand projection is useful but is not a general full-Jacobian-to-action or provider-effect qualification.

| Choice | Simplest viable/library alternative and integration cost | Decision and settling evidence |
| --- | --- | --- |
| Derivative actions | Existing Symbolica symbolic differentiation/hyperdual components plus faer forward/transpose multi-RHS factors; retain exact-entry/full-jet routes for their consumers | Prefer demand propagation first; compare direct contractions versus sparse pairs under original guard obligations. No dedicated reverse HVP integration established. [Symbolica AD](https://symbolica.io/docs/automatic_differentiation.html). |
| Strong root preconditioning | Existing PETSc GMRES/ILU and nonlinear Schwarz versus current KINSOL KLU/BlockFactor; stronger field/Schur or threaded unsymmetric factors only if needed | Operator, preconditioning matrix, split provenance and setup cost must be explicit. Schur `a11` is unsuitable for a zero A11 block; alternatives differ in fill/solves. Nonlinear Schwarz is not linear field splitting. [PETSc Schur choices](https://petsc.org/release/manualpages/PC/PCFieldSplitSetSchurPre/). |
| Dynamic actions/checkpoints | Pinned Diffsol in-place operator traits and existing IDAS JVP/adjoint owner; assembled reuse remains viable | Compare complete trajectories. Native checkpoint stepping is an integration control, not a replacement numerical algorithm. [IDAS 7.1.1 adjoints](https://sundials.readthedocs.io/en/v7.1.1/idas/Usage/ADJ.html). |
| Sparse responses/rank | Retain faer LU actions and existing dense SVD; optional SPQR/estimator with a different public method | Vendored SPQR is not default-enabled/qualified linkage or ownership. Only add a new rank product for a consumer that accepts its meaning; strict fill eligibility remains separate. |
| Scheduling/fitting | Existing MathService/session lanes and study frontier; stable experiment affinity versus unstarted dynamic dispatch | Avoid another scheduler/process framework. Qualify actual overlap, bounded residency and deterministic reductions; old wave maxima are insufficient. |
| Live output/read | Existing append/fence owner plus compact capture and actual decoder lanes | Moving encoding/reusing buffers can preserve per-sample flush. Cross-sample batching additionally changes latency/barriers; assess both before choosing. |
| Source/cache granularity | Actual typed delta and immutable dependency-index products; granular payload/protection sharing | Preserve complete dependency invalidation. Historical oversized protection grants motivate inquiry, not lower accounting or a larger default. |
| Compiler/profile | Current generic compiled evaluator, release dependency opt-level 3, dist FatLTO/one unit; PGO on genuinely hot artifacts | Compare actual consumed build. Include preparation/code size/failure paths; Rust PGO does not automatically optimize generated C++ or foreign libraries. [Cargo profiles](https://doc.rust-lang.org/cargo/reference/profiles.html). |
| SIMD/JIT/GPU/precision | Existing Symbolica batch/compiled backends and separately qualified realizations; conservative current route as control | Hardware SIMD is distinct from AD vectorization. Reopen failed JIT/subprogram candidates only on changed Second-order, stop-hook or ISA premises. GPU needs enough resident/batched work to amortize transfer/ownership. [Symbolica evaluation](https://symbolica.io/docs/numerical_evaluation.html). |

No license excludes a candidate. No universal solver winner, cache size, thread count, precision change or new dependency is selected by this review. Existing current-eligibility rules may themselves be changed when the target warrants it, through §10, rather than silently constraining the review's recommendation.

## 8. Priority, prerequisite order and efficient follow-up

**Consequence-based priority:** first correct the provider-effect bypass before widening reuse and correct checkpoint-limit enforcement; then remove full implicit expansion and large-block admission/work amplification; then address repeated callback/provider/frame/symbolic work and dependency barriers; then improve output/source/fitting/read throughput where exercised demand warrants it. Optional sparse rank, inner parallelism, PGO/SIMD/JIT/GPU remain comparisons, not assumed winners. A tiny callback cost can rank below a large-case refusal despite an easier implementation.

**Prerequisites differ from priority.** Demand semantics and provider composition precede demand-shaped buffer/admission tuning. Immutable provider binding must not erase current-effect checks. Successful factor/product association precedes order-upgrade reuse. Actual-overlap/team admission, aggregate work reservation and commit/failure semantics precede completion scheduling. An explicit live barrier/ordinal contract precedes microbatching. A rank method/cutoff and producer-storage choice precede a new sparse-rank or strict response claim. Fine-grained lexical dependency validity precedes incremental reference reuse. None of these dependencies requires an all-purpose registry or proof service.

Use existing diagnostics to choose the smallest revealing operation. Initial controls should finish in seconds where possible: vary one of total axes, demanded directions, support density, coupling/fill, block skew, output density or source-edit closure at a time. Inspect unexpectedly long activity, request cooperative cancellation and drain actual owners; a ten-minute run is not a prerequisite for these choices. One complete initial observation per case establishes work mechanisms; repeat only where variability can change a selection. Larger structures may be short-lived and more informative than a longer tiny trajectory.

Record complete same-qualified-result work: preparation, provider/components/RHS, factor analyses/numeric refactors/applications, callback gathers/copies, native iterations/steps/rejections, actual ready/idle/gate/output waits, encoding/append ACK and decode work. These are an existing diagnostic menu, not a requirement to instrument every counter before changing anything. Inclusive nested spans are not additive exclusive CPU time; reservation, live charged state, native fill and RSS are different quantities. Preserve cold/warm/prepared distinctions and failure/refusal/cancellation populations.

The next consequential decisions are the effect-safe selected provider/product contract, root transformation scope, actual block overlap policy and live batching boundary. Their tests can be bounded and local. Stronger factors, sparse rank, long-horizon checkpoint optimization, hardware realization and sustained-throughput follow-up need separate planning once their deciding premises are available. No detailed execution schedule is authorized here.

## 9. Verification and coverage limits

| Claim / inquiry | Evidence and result |
| --- | --- |
| Existing mathematical/provider/frame/source/block/output paths | **Implemented/source-inspected** at the baseline above; decisive source owners are linked in the evidence note. Three bounded inquiries plus a fresh independent assessment were reconciled by the coordinator. |
| Candidate library operations | **Interface-checked** where exact local source was inspected; Context7/current primary documentation is discovery. No new candidate was compiled or integrated. |
| Native short root control | Attempted `PSE_TEST_RESOURCE_STATE=<fresh private directory> just unit-native-capability-package pse-backend-native pse-backend-native/kinsol klu short_profile_contrasts_at_engineering_accuracy_preserve_original_roots --no-capture`. Command failed with exit 1; managed build exited 101 after compiler-cache `KeyError: SCCACHE_DIR` in two bindgen compilations. **Zero tests executed; zero passing numerical observations.** This is not a numerical failure or a measured solver comparison. |
| Required environment diagnostic | `just ready` failed with exit 1 before checks: the shared resource registry requires explicit fenced migration recovery. No shared registry was reset/migrated and no unrelated service was stopped. These environment repairs are outside this review. |
| Provider-effect cache reachability | **Implemented/source-inspected:** compiler admits provider-backed selection, runtime constructs it, both direct same-demand cache paths omit effect qualification, and projection exclusion covers only cross-demand reuse. AF19 records the source-established counterexample and test gap. No enclosing runtime counterexample was executed; final scientific-result consequence is unresolved. |
| Proposed corrections and speed benefit | **Proposed**, not Tested/Measured. No long simulation, broad integration/Clippy suite, full Maturin install or wheel build was run. Scientific accuracy was not changed. |

Historical Plan44/45 timings, work counts and candidate refusals were used only within their recorded scopes. They are not current measurements and cannot establish end-to-end speedup, observer overhead, a cache default or a native winner. A stopped diagnostic publisher in one exact-counter fixture does not establish ordinary telemetry overhead. Environment/provenance attestation work is excluded, with no deferred revisit obligation.

Unexamined breadth includes thermodynamic-package-specific hotspots, all class-specific native routes, large sustained workloads, general implicit/higher-index DAEs, device/distributed execution and full memory allocation completeness. These are coverage limits, not findings. Missing premises for selected remedies are stated at the responsible AF/RC row instead of inventing an implementation backlog.

## 10. Rule impacts and disposition route

These are **Proposed** target changes, not approvals. A future plan owns per-item decisions, disposition and implementation status. Existing plans keep their scope; no finding is marked scheduled or resolved here. Implementation-only improvements that preserve existing meaning (scratch reuse, prepared maps, borrowed cache lookup and exact lexical invalidation) need no invented rule change. Conditional rows activate only if the indicated alternative is selected.

| ID | Current rule / owner | Recommended change, scope and dependency | If retained unchanged |
| --- | --- | --- | --- |
| <a id="rc01"></a>RC01 | Blueprint §§7.3/14.3; `pse-kernels::ProviderRequest`, provider lift and derivative admission | Add actual selected/action product demands, capability/extent/effect semantics and composition; retain full physical inputs and full jets when consumed. Enables AF01–AF03/AF08. | Narrow outer actions continue expanding complete local jets or must explicitly refuse. |
| <a id="rc02"></a>RC02 | Blueprint §14.3 provider binding/current-effect and numerical-product contracts | Enforce immutable descriptor lifetime at actual attempt binding; separate invocation validity from completed numerical reuse; define atomic fill/borrow completion. AF08/AF09/AF13/AF14. AF19 is enforcement of the retained current-effect rule, not permission to weaken it. | Keep uncontrolled boundary checks/current effects and owned staging; fix AF19 under the existing rule. Equal coordinates alone cannot justify broader reuse. |
| <a id="rc03"></a>RC03 | Blueprint §§7.5/14.3/18.2 factor/product/reset and callback contracts | Explicit exact matrix/factor/first-product association, successful instantiation state and effect-safe assembled completion revisions. Preserve foreign-output shield and final original assessment. | Retain repeated factor/assembly work wherever compatibility/effects are not established. |
| <a id="rc04"></a>RC04 | Blueprint §§7.5/15/18.10; native presolve qualification | Distinguish root primal transformation/reconstruction from optimizer dual/KKT requirements; qualify each eligible reduction with original bounds/rows. AF04. | Root route remains unchanged; no blanket NLP presolve reuse. |
| <a id="rc05"></a>RC05 | Blueprint §§17.1/17.6/18.8; canonical wave/aggregate work/neutral reset | Dependency-local successful commit, deterministic reporting/failure, actual-live-set admission and one finite aggregate work policy; backend-qualified neutral reuse only. AF11. | Retain wave barriers; prepared maps/compact bindings and within-wave allocation reuse remain available. |
| <a id="rc06"></a>RC06 | Blueprint §15.4/§19.4 public scaled SVD rank/response meaning | If adopted, add separately named sparse QR/regularity/estimator product with method, scaling/cutoff, availability and storage; never replace SVD under its name. AF07. | Keep full spectrum semantics and current extent limits; independent sparse actions still improve. |
| <a id="rc07"></a>RC07 | Blueprint §18.8 and KINSOL/PETSc serial/team/process contracts | If justified, extend threaded unsymmetric root factors or reconstructible child-process execution at existing owners, with actual ABI/team/storage/callback/drain support. AF05. | Keep serial root/process gates; compare existing profiles and outer concurrency. |
| <a id="rc08"></a>RC08 | Blueprint §13.6 adjoint checkpoint population/step ownership | Retain the intended finite population rule; correct AF15's enforcement and native-call boundary without weakening it. A different checkpoint/replay policy would require a later concrete decision. | The current rule still requires a correction; post-call refusal cannot be represented as the configured pre-allocation population bound. This row records preservation, not permission to retain the defect. |
| <a id="rc09"></a>RC09 | Blueprint §§13.6/19.2 live-sample flush/ordinal/ACK contract | If batching is selected, state physical chunk membership and bounded visibility/flush barriers while preserving original sample ordinals, actual ACK and explicit detachment. AF12. | Keep per-sample flush; off-callback encoding and buffer reuse remain possible. No implicit loss/detach. |
| <a id="rc10"></a>RC10 | Blueprint §18.8 and `WorkAdmission::admit_storage` strict opaque-fill rule | Decide required strict sparse-response eligibility separately: improve producer bound/owner or explicitly revise policy after concrete comparison. Retain Local versus strict distinction and unknown conservatism meanwhile. AF16. | Strict faer response remains unavailable; symbolic reuse must not imply eligibility. |
| <a id="rc11"></a>RC11 | AGENTS CPU policy; blueprint §14.3 realization and generic compiled arithmetic contract | Only if selected: explicit ISA/FMA/precision/JIT/GPU support and equivalence/lifecycle contract. PGO/profile tuning alone need not change scientific meaning. AF10/U20–U22. | Keep generic arithmetic/ISA and current qualified realization; no universal hardware adoption. |

No ADR is drafted and no current rule is silently changed. The review's independent architectural judgment is **Revise**; the next plan-creation step can accept, reject or refine these recommendations and conditional routes without confusing that choice with implementation or qualification.
