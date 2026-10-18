# Post-Plan44 native solver inquiry

Supporting inquiry for [the principal review](../../reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md). The principal owns the integrated judgment and follow-up; this file records a bounded source/library assessment, not a competing disposition ledger.

## Baseline, authority and effects

Examined `/home/paul/pse-arrow` at `91fd915134f99edeb8ef00368096f84761960d95` on 2026-10-18. Judgment basis: core design principles 3.4, efficient architecture heuristics 1.0, ProcessSimulator 1.5 and its functional target, selected by `docs/design_review/design_principles/standard.toml`. Existing ADRs, defaults and exact pins describe the current design; they do not constrain a target-design recommendation. Relevant lenses are AP-07, DP-09/13/15/20/22, H5/6/8/9/12/14/17/28 and PS-07/09/10. This is bounded source and retained-artifact inquiry, not implementation, new runtime qualification or whole-product acceptance.

Loaded shared worker/library-research contracts, native-solver-libraries, Context7, design-review and process-simulator skills. Read Plan44b/d/e/f and relevant integration/backend source. Did not consult `docs/library-utilization.jsonl` or `docs/capability-maps`. No builds, services, tests or numerical probes were run. Only scratch write: this file. No production/docs/shared-skill edits, commits, pushes or delegation.

Preserved pre-existing dirty `.envrc`, `.python-version`, `docs/dev/agent-environment.md`, `python/pse/tests/test_native_workflow.py`, `scripts/pse_env.py`, and untracked `scripts/setup-env`.

Repository source paths below are relative to the repository root; corpus paths are relative to `/home/paul/.local/share/library-skills/skills/native-solver-libraries/content/corpus/`. Source line references locate inspected implementation; enduring rule citations use stable section identifiers.

## Conclusions and Plan44 reconciliation

| Plan44 conclusion | Current disposition | Reason and next target |
|---|---|---|
| F06 / 44d D02: cross-call KLU symbolic work remains | Retained, unresolved remedy | Exact 7.1.1 initialization repeats symbolic analysis on nontrivial calls. Allocation reuse and accepted starts do not establish factor reuse. Investigate complete symbolic ownership at representative sparse shape. |
| 44d D04 / O05: explicit KINSOL/recycle profiles, no universal winner | Retained | Setup interval trades residual work for derivative/factor work; recycles use Anderson controls, not Newton setup interval. Existing tiny observations cannot choose a general policy. |
| F01-F03 / 44b B01-B03: optional-rank preflight, selected forward directions, one fresh final product | Already implemented; retain controls | Source carries selected native width with complete physical values and independent final assessment. Do not reschedule as missing. |
| O01 / 44b B04: keep lazy defaults | Retained | Native callbacks do not guarantee future derivative demand. Rejected value trials can make eager derivative work wasteful. |
| O02 / 44b B05: conditional exact forward quadratures | Already implemented; larger crossover unresolved | Combined already-forward IDAS demand avoids adjoints; integral-only and before-effect refusal retain adjoints. Dimension crossover remains consumer-specific. |
| Output-density/native-step interpretation | Retained, with important comparison correction | Matched unscheduled density cases show identical work; resweep ode25 includes a schedule and extra physical axis, so comparing it directly to resweep ode5 confounds density and scheduling. |
| Selected adjoint parameter-coordinate demand | New candidate investigation | Forward selection does not narrow full adjoint parameter quadratures/products before consumer projection. No measured material benefit established. |
| Long/stiff/sparse dynamics and checkpoint behavior | Retained future investigation, sharpened | Compare workload shape and complete recording/replay/backward state rather than duration or solver rankings. |
| Other class routes, updates and warm state | Retained; avoid duplicate proposals | HiGHS compatible updates/basis retention already exist. Tiny cross-class cases establish route behavior, not backend superiority. |

No new scientific correctness defect was established. F06 is confirmed repeated work with an unresolved beneficial integration route. The selected-adjoint proposal is a source-backed opportunity, not a newly qualified performance defect.

## 1. KINSOL cross-call symbolic preparation

**Interface-checked current behavior:** `crates/pse-backend-native/src/kinsol.rs:1241` (`Session::replace`) retains allocation and exact sparse layout but clears callback setup. `:1677` configures `KINSetMaxSetupCalls`; `:1680` explicitly configures `KINSetNoInitSetup(..., 0)`. This is sound: changed coefficients/equations do not inherit valid numerical factors merely because support and allocation match.

Pinned SUNDIALS 7.1.1 corpus chain:

- `sundials-sys@0.6.2/vendor/src/kinsol/kinsol.c:1578`: initial-guess acceptance precedes linear initialization; `:1603` calls the linear initializer for nontrivial applications.
- `.../src/kinsol/kinsol_ls.c:1140`: calls `SUNLinSolInitialize`.
- `.../src/sunlinsol/klu/sunlinsol_klu.c:231`: initialization marks `FIRSTFACTORIZE=1`.
- Same file `:255`: next setup frees/rebuilds symbolic analysis and performs numeric factorization. Subsequent setups within that application refactor and inspect conditioning.
- Same file `:139-168`: `SUNLinSol_KLUReInit` frees both symbolic and numeric objects.

The repository's instrumented `vendor/sundials-sys/vendor/src/sunlinsol/klu/sunlinsol_klu.c` retains that lifecycle (`SUNLinSolInitialize_KLU` at line263, symbolic issued counter at294). Instrumentation does not establish a different reuse guarantee.

Upstream: [exact KLU source](https://github.com/LLNL/sundials/blob/v7.1.1/src/sunlinsol/klu/sunlinsol_klu.c), [exact KINSOL source](https://github.com/LLNL/sundials/blob/v7.1.1/src/kinsol/kinsol.c), [exact KINSOL linear source](https://github.com/LLNL/sundials/blob/v7.1.1/src/kinsol/kinsol_ls.c).

**Judgment:** F06 remains AP-07/H12 work-repetition evidence for repeated sparse applications. Plan44d's 1.8–10.7 microsecond symbolic intervals are once-only tiny-case observations including hooks; they do not establish an end-to-end benefit sufficient to justify a maintained replacement.

**Concrete target:** compare stock KLU with a narrowly owned SUNLinearSolver binding retaining only symbolic analysis under complete CSC structure and all relevant Common analysis settings. Fresh numerical setup is mandatory per application. Compare an eligible existing Krylov/block-preconditioned route before assuming direct symbolic retention is the best complete alternative. KINSOL remains nonlinear iteration/globalization owner.

**Smallest discriminating probe, not run:** one retained nontrivial sparse application with unchanged CSC and significantly changed coefficients; include accepted versus ordinary start, changed ordering/Common settings, changed pattern, singular factorization and recovery. Count symbolic/full-numeric/refactor/substitution separately and retain original residual qualification. Use enough sparsity/fill to expose preparation; extending a four-variable run's duration does not supply that shape.

**Adverse cases:** stale analysis settings; numeric pivot/refactor failure; changed support; cancellation/drain; accounting and repeated release; accepted starts falsely presented as setup reuse.

**Impact:** native linear integration/reuse ownership and blueprint §18/§23 explanations. No public scientific API change is intrinsically necessary. Suppressing initialization or retaining prior numerical validity would alter the current safety contract and is unsupported here. No factor crosses into faer's distinct IFT problem.

## 2. Selected forward and independent fitting final products are completed

**Implemented source:** Diffsol holds complete physical values separately from `axes`; `crates/pse-backend-native/src/dynamics/integrator.rs:112` makes native width `axes.len()`, while `:735` updates selected coordinates in the complete vector. Corpus `diffsol@0.16.2/src/ode_equations/sens_equations.rs:26,111,278` derives sensitivity dimensions from `rhs().nparams()`. The selected-coordinate equation addresses actual library work; truncating the physical value vector would not.

IDAS selects explicit physical axes at `dynamics/idas.rs:1691`, supplies analytic sensitivity residuals at `:779`, and configures selected initial vectors and sensitivity error control together. Fitting's `workflow/fitting/oracle.rs:1093` creates a fresh final oracle independent of trial-cache state, after optional-rank preflight. Plan44b B01-B03 own completed refusal/fallback behavior and targeted evidence.

These mechanisms should remain regression controls, not proposed new work. Ordinary public Forward stays full, while a missing selected direction is unavailable rather than zero.

Exact Diffsol archive `.cargo_vcs_info.json` identifies commit `dc30fee49e692c65577b11a1181bfca62f5ebf42`, path `crates/diffsol`. [Exact sensitivity equations](https://github.com/martinjrobins/diffsol/blob/dc30fee49e692c65577b11a1181bfca62f5ebf42/crates/diffsol/src/ode_equations/sens_equations.rs). A guessed `v0.16.2/src/...` URL did not resolve via the web reader; use the archive's actual source coordinate.

## 3. New candidate: consumed adjoint parameter directions

**Interface-checked amplification:** `workflow/integrated.rs:194` clears forward-axis selection on non-Forward requests. `IntegratedExperiment::gradient` (`:333`) receives complete native gradients and maps all bindings; `window_gradient` (`:552`) returns the full anchored window gradient. `workflow/shooting.rs:1587` then projects window columns through `p.variable`.

Diffsol `Shared::new`, `dynamics/integrator.rs:829`, selects all integration parameters outside Forward. IDAS `adjoint_init` (`dynamics/idas.rs:1776`) and backward (`:2885`) use `integration.len()` for parameter quadratures. Therefore parameter-dependent products, quadrature state and storage can precede free/consumed-coordinate projection.

**Proposed target:** express selected adjoint parameter-gradient coordinates independently from state adjoints and forward directions. Keep complete physical values; reduce parameter quadrature/product/result scope where the consumer does not need the omitted coordinates. The state adjoint remains necessary and may dominate cost. No elapsed or material-capacity benefit has been measured.

**Smallest probe, not run:** gradient-only fit and integral-only shooting with one consumed free parameter and zero/four additional nonzero coupled fixed inputs; then add a selected scheduled interval and anchor. Compare full and selected gradient coordinates at unchanged science settings. Observe parameter-product width, quadrature workspace/evaluations and initial corrections. Check direct dependence, tied/scaled binding sums, schedule mapping and independent physical derivatives. Empty selection must avoid unconsumed parameter derivative work without corrupting values.

**Impact:** internal gradient-coordinate contract, initial/scheduled/anchored mapping, result projection and admission owners; blueprint §13.6/§19 explanation. Public adjoints can remain full, so no public API change is inherently required. Durable framing changes only if external shape/meaning changes. Do not use forward-axis fields to silently reinterpret a distinct adjoint contract.

## 4. Forward quadratures: implemented route, unresolved crossover

`workflow/shooting.rs:896` selects exact forward quadratures only when terminal/path rows already need Forward, integrals exist, and all windows use IDAS. Every window is admitted before effects. `dynamics/idas.rs:2254` supplies an exact callback to `IDAQuadSensInit`, explicit vector tolerances and error control; `:2281` retrieves the endpoint product. Each window is reduced immediately to the existing integral-gradient layout. Integral-only requests retain adjoints. After work starts, failure/cancellation does not launch another route.

[Pinned upstream FSA documentation](https://github.com/LLNL/sundials/blob/v7.1.1/doc/idas/guide/source/Usage/FSA.rst) describes callback and initialization requirements. Null callbacks permit internal difference quotients and would not establish this exact derivative contract.

**Proposed larger investigation:** vary consumed controls, integral channels and windows independently while retaining one physical objective and accuracy. Count actual Forward/backward/replay requests, selected direction width and simultaneous native/result storage. Preserve independent physical gradients and before-effect refusal controls. State size, channel count and sensitivity error control matter alongside parameter count; there is no universal few/many-control crossover established.

**Unresolved alternative:** contract a weighted integral objective before derivative integration. Stock quadrature sensitivities inherit primal quadrature shape, so this may require an explicit composed functional binding rather than buffer resizing. Keep required ordinary quadrature values/independent checks and assess added machinery before adoption.

**Impact:** shooting product selection, blueprint §13.6/§19.4 and any adopted ordinary-simulation integral-sensitivity API. Hybrid/reset derivative refusals remain explicit unless separately expanded. No new policy is selected here.

## 5. Dynamics: output density, stiffness and linear shape

Read actual `*-observations.jsonl` under both requested case directories. Matched density results:

| `build/p44-density-22s_40tp/cases` | Steps | Linear setups | Nonlinear iterations | Retained axes |
|---|---:|---:|---:|---|
| compact-diffsol-ode-5 |23|9|69|[0,1]|
| compact-diffsol-ode-25-unscheduled |23|9|69|[0,1]|
| compact-idas-ode-5 |16|16|17|[0,1]|
| compact-idas-ode-25-unscheduled |16|16|17|[0,1]|

Thus denser output did not add native steps in these controls. Resweep `compact-*-ode-25` includes scheduling and axes `[0,1,2]`; it is not a matched pure-density comparison with resweep ode5.

Resweep smooth/stiff decay reports Diffsol34/49 steps and IDAS20/38. These are different algorithms/counter meanings, not a backend ranking. A changing IDAS `cj·M-J` Newton matrix can require fresh numeric factors even when physical derivatives are constant.

**Proposed concrete target:** choose numerical algorithms and direct/Krylov/preconditioning profiles for stiff/sparse versus nonstiff shape. Existing Diffsol explicit methods are candidates for genuinely nonstiff problems; do not pay implicit factors automatically. `dynamics/layout.rs:176` and `idas.rs:1932` explicitly refuse IDAS block-factor preconditioning: a concrete integration question if existing Jacobi/unpreconditioned routes prove inadequate.

**Smallest probe, not run:** modest sparse stiff physical model with independent conservation and output checks; vary coupling/fill and stiffness while holding the scientific target fixed. Compare existing eligible linear profiles, actual setup reasons, nonlinear/error-test failures and physical progress per step. Follow with longer horizons only if those reveal rejection/setup growth or stalled scientific progress. Failure limits alone do not establish ordinary-workload feasibility.

**Impact:** numerical profile/capability and native preconditioner ownership; blueprint §18 if new consumed support is adopted. Ordinary tolerances and original physical checks stay intact. A changed default would be an explicit policy/rule impact, not an incidental tuning decision.

## 6. Adjoint checkpoints and sample demand

Diffsol retains a checkpointed solver per scheduled segment before backward traversal (`dynamics/integrator/adjoint.rs:355`). Its host recorder exists because required reset metadata is crate-private; the library still owns replay and adjoint integration. IDAS retains one segment's checkpoints and replays earlier segments (`dynamics/idas.rs:2861`). The two complete alternatives exchange retained memory for replay work.

Both already skip zero-weight observations: Diffsol `integrator/adjoint.rs:389`, IDAS `idas.rs:2910`. Do not propose that as a missing optimization. Nonzero observation jumps legitimately restart/consistently initialize backward state.

**Smallest probe, not run:** one versus several scheduled segments, terminal-only versus several nonzero interior observations, two checkpoint intervals. Separate recording, replay and backward work and simultaneous solver/history storage. Preserve algebraic consistency, sample-side schedule convention, initial corrections, failure invalidation and explicit memory refusal.

**Library investigation:** determine whether the pinned checkpoint API or a narrow upstream extension can remove the host metadata/recording gap. [Exact checkpoint source](https://github.com/martinjrobins/diffsol/blob/dc30fee49e692c65577b11a1181bfca62f5ebf42/crates/diffsol/src/ode_solver/checkpointing.rs), [exact adjoint equations](https://github.com/martinjrobins/diffsol/blob/dc30fee49e692c65577b11a1181bfca62f5ebf42/crates/diffsol/src/ode_equations/adjoint_equations.rs), [IDAS adjoint documentation](https://github.com/LLNL/sundials/blob/v7.1.1/doc/idas/guide/source/Usage/ADJ.rst). Current Diffsol adjoint documentation is discovery context: https://github.com/martinjrobins/diffsol/blob/main/book/src/solve/adjoint_sens.md.

**Impact:** checkpoint ownership and admission/replay contracts, dependency/API changes if adopted, blueprint §13.6/§18. Distinguish native checkpoint history from a durable reusable workflow session; no cross-trial checkpoint reuse is established.

## 7. Other native classes

Plan44e C06 exercises distinct LP/MILP/MINLP, Uno/PETSc and assembled QP/conic routes. `crates/pse-backend-native/src/highs.rs:674` already updates compatible costs/bounds/matrix/Hessian with allocation/basis retention; numeric factor validity remains HiGHS-owned. Adding warm starts/updates without examining this owner would repeat implemented functionality.

Future investigations should preserve class representation: coefficient updates/basis behavior for repeated LP/QP; Clarabel cone/update restrictions; SCIP structural expressions/bounds; analytic JVP and preconditioner shape for large square systems. Skill source routes `linear.highs.md`, `conic.clarabel.md`, `minlp.scip.md`, `root.kinsol.md` describe exact interfaces and limits. Clarabel in-place updates reject active presolve/chordal decomposition and dropped structural zeros; that tradeoff matters for repeated cone models. HiGHS continuous convex QP does not establish MIQP or nonconvex-QP support.

These are eligible investigations, not confirmed omissions or cross-class winners. Exact class/backend version/configuration qualification and original objective/feasibility/integrality/certificate semantics remain necessary. This inquiry did not perform full PETSc/Uno capability research or new class probes.

## Artifact uncertainty and adverse evidence

`build/p44-resweep-t2zaldrm/cases/compact-root-banded-8/diagnostics/detail.json` explicitly records incomplete raw streams:516 dropped root-evaluation records and5 dropped native-owner records. Complete numerical reports may support scoped counts; missing raw records cannot establish complete callback populations or exclusive attribution.

Returned resweep coupled-root observations report99/217 evaluations,21/38 iterations and3/5 KINSOL setup refreshes for dimensions4/8. Setup refreshes are explicitly not factorization counts. Several diagonal/banded cases have no returned native report; that absence is not zero native work or an implementation failure. Whole harness seconds include setup/observation and are not native solver times.

No universal backend/profile ranking, long-horizon physical qualification or global speedup follows. Independent backward-error/original assessments remain necessary even when library factors and native status succeed. Library integration should remove complete generic work where the consumed contract fits, while preserving the specific adapter gap and one numerical iteration owner.

## Context7 calls and outcomes

Used the exposed `mcp__context7__resolve_library_id` and `mcp__context7__query_docs` tools. Exact call inputs:

1. Resolve `{libraryName:"SUNDIALS", query:"KINSOL reuse Jacobian setup KINSetNoInitSetup IDAS adjoint quadrature checkpoint sensitivities"}`. Outcome: selected `/llnl/sundials`, High reputation,14734 snippets, score71.25.
2. Resolve `{libraryName:"Diffsol", query:"Rust BDF adjoint checkpoint sensitivity quadrature selected parameters"}`. Outcome: selected `/martinjrobins/diffsol`, High reputation,396 snippets, score61.06.
3. Query `/llnl/sundials`: `KINSOL KINSetNoInitSetup KINSetMaxSetupCalls KLUReInit SUNLinSolInitialize reuse symbolic factorization across solves`. Outcome: docs for no-initial-setup control, default maximum setup interval10, KLU reinit freeing/resetting symbolic/numeric preparation, and within-application initial factor/refactor lifecycle. Sources were upstream main docs; exact7.1.1 local source checked before transferring claims.
4. Query `/llnl/sundials`: `IDAS adjoint checkpointing IDAAdjInit IDAQuadSensInit user supplied callback quadrature sensitivity initial conditions`. Outcome: initialization ordering, null-callback difference quotients, Ns sensitivity vectors, checkpoint interval/Hermite-polynomial choices and native storage discussion. Returned snippets included different `IDAAdjInit` argument spellings, reinforcing exact-header verification rather than copying current snippets.
5. Query `/martinjrobins/diffsol`: `selected parameter sensitivities nparams forward sensitivities adjoint checkpointing solve_adjoint gradients`. Outcome: forward sensitivity and checkpointed-adjoint documentation. Some snippets used older/incompatible-looking public spellings (`OdeProblem`, `RK45`, differing builder forms). These were discovery only; exact0.16.2 corpus/source and repository adapters determine the applicable contract. No public selected-axis API guarantee was inferred from search results.

Web verification opened the pinned SUNDIALS KLU source successfully. The guessed Diffsol tag/source URL returned an internal reader error; archive `.cargo_vcs_info.json` supplied the precise source commit/path. No library eligibility or absence claim relies on that failed URL.

## Evidence boundary and unfinished work

**Interface-checked:** cited native/adapter contracts and source control flow. **Implemented:** mechanisms present in current source. Historical Tested/Measured observations retain Plan44b/d/e/f's exact scope; they were not rerun here. All proposed probes are **not_run**, with no new pass/failure count. No scientific setting changed.

The coordinator owns disposition, review publication and any subsequent authorization. Most useful sequencing is: preserve completed work; investigate consumed-adjoint amplification and representative sparse preparation; use resulting native work/failure/memory facts to select dynamics/checkpoint/class experiments; consider latency only after complete alternatives and independent scientific checks are fixed.

Memory navigation used `MEMORY.md:2297-2306` to locate native-reference qualification boundaries, subsequently checked against current source. Associated rollout ID: `01a0d1d1-c2da-7b50-a31e-c7821a04ea7d`. No memory-derived historical runtime claim is promoted to current qualification.
