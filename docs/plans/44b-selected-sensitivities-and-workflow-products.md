---
title: Selected sensitivities and workflow products
status: in-progress
date: 2026-10-17
adrs: [ADR-0197]
review_sources: [docs/design_review/reviews/design_review_pipeline-performance-optimization_2026-10-17.md]
doc_owner: docs/plans/44-pipeline-performance-optimization.md
doc_topics: [current-work, numerical-execution, design, libraries, validation]
---

# Plan 44b: Selected sensitivities and workflow products

## Purpose and foundation assessment

This companion owns F01–F03/F09 and O01/O02 from the [source review](../design_review/reviews/design_review_pipeline-performance-optimization_2026-10-17.md). [Plan44](44-pipeline-performance-optimization.md) owns dispositions and accepted RC02. **At authoring — Proposed:** all packages, interfaces and tests below; the specified-change checkpoint records subsequent implementation.

The existing fit oracle, integrated experiment/window binding, native IDAS/Diffsol adapters and original trajectory checks are the foundations. `workflow/fitting/oracle.rs` already distinguishes independent final assessment, demand upgrades and partial products. `workflow/integrated.rs` converts complete physical parameter bindings and gathers responses. Native `dynamics::{Profile,Sample}` currently implies full parameter sensitivity width. Preserve those responsibilities; reducing the number of native vectors alone cannot establish the selected-direction contract.

The short fits measured work amplification and split final integrations, not a backend winner. Complete selected integration-purpose streams supply useful current baselines; ordinary snapshots can be partial. [44a](44a-invocation-coverage-and-diagnostic-readiness.md) fixes their separate recorder/publication defects. Numerical work can be diagnosed from valid existing facts without waiting for all observer work.

## B01 — Preflight optional response/rank work

Extract deterministic response/rank admission before requesting derivative work: free/measurement dimensions, arithmetic overflow, local dense cells, profile `max_cells`, worker numeric/scratch extent and the separately accounted structural matching stack. Known refusal must not launch an identifiability IVP. A gradient-only solve may still need derivatives for optimization; the preflight governs the optional final response/rank product, not unrelated necessary work.

Keep known capacity limits distinct from current pool availability. Decide the actual lifetime of optional matrix reservations relative to integration; holding a matrix allowance over an IVP must not accidentally increase simultaneous live memory or introduce a refusal where the sequential route fits. Reserve at the actual effect boundary with sound admission of the composed peak. Do not infer numerical rank before computing it or enlarge allowances to make a test pass.

Acceptance uses a small admitted fit, the declared 12-parameter rank-refusal control, checked extent overflow/worker refusal, and numerical rank deficiency. An independently assessed candidate remains available when optional diagnostics refuse; singular rank is not a resource refusal. Selected integration-purpose records distinguish necessary optimization from avoided final response work.

## B02 — Complete values and explicitly selected directions

Define one internal direction/seed layout consumed through preparation, integration, initial-state derivatives, scheduled segments, response gathering, retained products and memory estimates. Complete physical parameters, contract parameters, selected directions and shooting anchor coordinates are different layouts. Preserve binding identities, nonunit conversions, tied-binding addition and declared ordering. A missing direction is unavailable, never an invented zero. Empty consumed selection performs value integration without allocating sensitivities; ordinary full Forward continues to request all admitted axes.

Resolve backend realization before fixing a neutral wrapper. Pinned IDAS can select `Ns` with an explicit callback; host allocation/callback indexing and sensitivity error control must migrate together. Pinned Diffsol sensitivity equations derive width/workspaces from `rhs().nparams()`, while `set_params` carries complete values. Investigate a selected-coordinate equation binding retaining complete immutable physical values versus a supported native augmentation or narrow library extension. A smaller host width or truncated value vector is unsound. Use the native-solver and Symbolica/faer skills, Context7 for current documentation and exact pinned source/compile controls; stale utilization records/capability maps are not evidence. A necessary dependency change follows exact-pin and affected-test policy, not a wholesale update.

Migrate both eligible native backends and actual fitting/window consumers, including public projections if their shape/meaning changes. Do not silently implement selection on one backend while reporting equivalent work on the other. If one realization remains unresolved, retain that explicit incomplete branch rather than silently dropping a supported backend. Follow boundary/registry/codegen and decision routes for real external-contract changes; no new public selection API is needed merely to pass internal consumed directions.

Acceptance pairs zero/four added fixed inputs with the review's unchanged linear answer, then adds a genuinely coupled state, **nonzero** fixed input, nonunit scales, tied bindings and one selected scheduled interval. Check initial and endpoint responses against analytic cases and independent derivative controls. Count actual selected axes/native work at matched scientific settings; changed sensitivity error-control populations may change steps, so exact step equality is not universal. Preserve original accuracy and refused unsupported combinations.

## B03 — One fresh admitted final product

After B01 admission, request one fresh final value-plus-response product when responses are required and supported. Original assessment and rank analysis consume its completed products instead of separate successful IVPs. The final oracle remains independent of optimizer trial caches and native success status. B03 may use the current full-direction layout before B02 lands; the operations must compose when selection is integrated.

A derivative failure may fall back to fresh value-only assessment only if execution remains live and the work is admitted. Cancellation/deadline prohibits starting that fallback. Preserve independently completed checked values where available; do not promote an incomplete derivative upgrade or stale trial prediction into an assessed answer. Rank refusal retains its reason and the candidate, and response/statistical products retain their governing permission.

Acceptance contrasts successful final response/rank, deterministic rank refusal, runtime derivative failure, numerical rank deficiency, cancellation before fallback and failed original assessment. Complete selected records should show one fresh successful final integration where two were previously used. Do not require a fixed whole-fit integration count across every native optimizer trajectory.

## B04 — Conditional trial-product scheduling

Investigate O01 through the existing native callback demand and point cache. Compare lazy upgrades with explicitly selected value-plus-response preparation when the consumer is known to need it. Preserve the default lazy behavior and completed weaker products until a stronger product succeeds. Changed weights, original checks and failed upgrades retain current semantics.

Use short accepted/rejected line-search sequences, few/more selected seeds, gradient-only adjoints and exact/Gauss–Newton curvature routes. Record which integrations/upgrades are avoided and which rejected trials acquire unused derivatives. Select a bounded consumed-product policy only when it removes work without unjustified overfetch; a supported no-change conclusion is valid. No host optimizer iteration, automatic global eager default or universal timing winner is proposed. A broader change to blueprint §19's default laziness would be an additional rule decision, outside the already accepted RC02.

## B05 — Qualified conditional quadrature sensitivities

Accepted RC02 permits investigation, not a selected implementation. Compare integral-only and combined terminal/path-plus-integral shooting requests, few/more controls and one/multiple windows. When forward directions are already needed, an exact forward quadrature-sensitivity product may avoid another checkpointed forward/backward operation; integral-only/many-control cases may favor the retained adjoint. Preserve the existing same-point lazy/upgrade cache and window/parameter/anchor layout.

Check pinned `IDAQuadSensInit`/`IDAGetQuadSens` and supplied derivative callbacks, direct parameter dependence, initial quadrature/anchor derivatives, accumulated offsets across scheduled reinitialization and quadrature error control. A null callback's internal finite differences are not an exact derivative implementation. Qualify physical gradients and required products before selecting a route; do not infer benefit from API availability.

Preserve current IDAS event/reset sensitivity refusals. Reset-bearing value integration remains supported under its existing scope; hybrid fitting/shooting derivatives do not become mandatory new scope. Include typed refusal controls instead of silently broadening support. Ordinary simulation integral-sensitivity APIs also stay outside this bounded route unless a concrete separately decided consumer requires them. Publish the actual selected realization through RC02's blueprint §13.6/decision route before adoption. Retain adjoints and a no-change branch if no qualified beneficial route is established.

## B06 — Ordered original trajectory checking

Remove repeated coverage-grid `.position(time)` searches in `modeling/dynamics_checks.rs`. Carry original ordinals through the ordered retained prefix and merge a separately retained endpoint once, replacing a coincident point with its correct endpoint inputs. Preserve Full/Compact behavior, partial coverage, final mode/segment identity and every original physical/conservation check. An off-grid endpoint is not a fabricated regular sample.

Acceptance covers 5/25 outputs, an increased but still short grid, off-grid/coincident endpoint, scheduled change/event, partial prefix and early termination. Full and Compact checks agree where their requested products overlap; Compact's existing explicit sensitivity refusal remains. Do not introduce native stepping merely to satisfy an output/check ordinal. This change is independent of B01–B05.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| B01 | Existing rank/worker/structural admission contracts | Known optional refusal before derivative IVP; candidate-preserving refusal tests; remove late duplicate admission | done |
| B02 | Exact IDAS/Diffsol realization and consumed layout decision | Selected directions across initial/scheduled/native/result/admission owners; full-Forward preservation and coupled/scaled tests; delete full-width assumptions in migrated paths | done |
| B03 | B01 implemented; existing final oracle; B02 only where selected layout is consumed | One fresh admitted final product, live-only value fallback and original permission; remove duplicate successful final integration | done |
| B04 | B02/B03 relevant product semantics; E03 accepted/rejected trial controls | Supported conditional scheduling decision and any selected implementation; retain default lazy route and failed-upgrade behavior | open |
| B05 | B02 window/direction contract; accepted RC02; exact native capability and E03 crossover controls | Qualified conditional forward quadrature route or supported retained-adjoint conclusion; preserve hybrid refusals | open |
| B06 | Existing coverage/endpoint/check semantics | Ordered sample checking and endpoint merge; Full/Compact/partial/event tests; delete repeated grid lookup | done |

## Verification and checkpoint

Package tests run with their implementation and selected compile checks. Feed actual work/product facts to 44e rather than embedding benchmark thresholds in scientific APIs. [44f](44f-integrated-acceptance-and-follow-up.md) owns assembled fitting/shooting/dynamic checks and final quality. At authoring, all B packages were open; document creation itself implemented no numerical remedy, backend selection or API change.

## Specified-change checkpoint

**Implemented and targeted Tested:** B01 preflights deterministic rank cells, dimensions, overflow, numeric/scratch and structural-stack allowance before optional response work, reserving rank memory after the IVP. B03 performs one fresh final response/value evaluation independently of optimizer cache, reuses it for rank, preserves value-only fallback after derivative failure, and starts no fallback after cancellation. B02 propagates consumed physical axes through Diffsol/IDAS, initialization, schedules, anchored/shooting gathering, reduced sample/native admission and retained reports. Free semantic inputs resolve before fitting preparation admission; typed replay preserves that request or explicit physical axes. Ordinary public Forward remains full. Public response rows include `parameter_column` so scheduled intervals stay distinct, and dynamic profile framing is version10 (ADR-0197). Complete fixed values, tied scaled contributions, independent checks and practical accuracy remain. B06 merges ordered sample/coverage ordinals and a separate endpoint without repeated grid search. Native selected/full/empty, coupled/scaled/scheduled/anchor, public projection/replay, rank/fallback/cancellation and ordered endpoint controls pass in the coordinator's scoped selection. B04/B05 remain unrun; no quadrature-sensitivity choice or eager-trial policy was adopted.

The coordinator owns command/result totals and the enclosing acceptance boundary. These package results do not close the 44e campaign or 44f qualification.
