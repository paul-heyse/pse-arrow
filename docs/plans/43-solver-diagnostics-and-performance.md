---
title: Solver diagnostics and performance
status: in-progress
date: 2026-10-17
adrs: [ADR-0192]
review_sources:
  - docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md
scenario_sources:
  - docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md
doc_owner: docs/plans/43-solver-diagnostics-and-performance.md
doc_topics: [current-work, design, preparation, libraries, validation]
---

# Plan 43: Solver diagnostics and performance

## Purpose, authority and baseline

This standalone series captures the [integrated target review](../design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md), its [complete recommendation crosswalk](../design_review/evidence/solver-diagnostics-and-performance-target-2026-10-17/recommendation-crosswalk.md) and [capability evidence](../design_review/evidence/solver-diagnostics-and-performance-target-2026-10-17/capabilities-and-evidence.md). It also retains relevant detail from [external review R1](../external_review_numerical_diagnostics_1.md) and [external review R2](../pse-arrow-solver-performance-review.md), using the integrated review's corrected positions rather than restoring rejected premises.

The target is a best-in-class simulator with explainable, effective execution across edit/re-solve, studies, roots/optimization, recycles, dynamics, fitting/shooting and extension. Value changes retain structural work whose premises remain valid. A responsibility remains locally compilable/testable with its actual dependencies. Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5 supply the design basis. Current contracts remain binding during execution until their proposed changes receive the decisions recorded below.

**Interface-checked:** reference documents, decisive source and current contract owners at `9cf67375cdf6d0b06c2dc51bc10e1ff72b9a439c`, inspected 2026-10-17. The review and its evidence plus the two supplied external reports were untracked inputs at authoring start. **At document authoring — Proposed:** all product work, investigations and acceptance below. No product implementation, native build, simulation, dependency adoption or scientific measurement was performed while creating this series. Document creation does not authorize product execution. The operator accepted RC01–RC11 as written on 2026-10-17; bounded investigation routes retain their stated limits and do not select unknown winners.

Existing plans are outside assessment and updates. No scope is transferred from them, no status or disposition is reconciled, and no supersession is enacted. Read-only current contracts and source provide the baseline. New-series publication/lifecycle registration exposes only these documents. The original review/reports remain unchanged.

## Combined target and contracts

The present scientific/observational separation is retained. Authored definitions and providers own physics; math owns selected products and transformations; native libraries own numerical iteration/globalization/factors; strategy owns declared starts/routes/spent work; original/workflow assessment owns scientific permission; observation/capture owns optional copied facts; pure interpretation owns explanations; existing artifact owners govern explicit retained evidence.

The combined design has three diagnostic products: ordinary bounded live observation, explicitly selected detailed collection, and derived analysis. It adds no universal solver event authority, second work ledger, generic cache, scheduler, project AD engine or bespoke eigensolver.

| Shared contract | Authoritative plan explanation / production owner | Consumer obligation |
| --- | --- | --- |
| Producer/attempt/event identity, activation, metric scope/coordinate/norm/coverage | [43a](43a-operation-evidence-and-coherent-live-diagnostics.md); producer and `pse-diagnostics` owners | No mixed callback populations, zero-for-disabled values, inferred iteration tuples or summed overlapping elapsed time |
| Coherent current/recent checkpoint versus detailed record stream | 43a; native extraction in [43c](43c-native-methods-reuse-and-matrix-boundaries.md) | History-off still permits current progress; finite decimation never claims complete capture |
| Complete invocation and outcome endpoints | 43a; actual workflow/benchmark/result owners | Qualification, terminal elapsed, caller response readiness and each product's availability remain distinct |
| Selected derivative/region/projection contract | [43b](43b-mathematical-products-and-evaluator-efficiency.md); `pse-math` and providers | Preserve lazy obligations, independent coordinates, aliases, original source/failure and complete successful products |
| Native reset/reuse and transformation witness | 43c; native/structural owners | Allocation/pattern/classification/scaling/numeric-factor/start reuse have separate premises; original checks remain |
| Dynamic advance/interpolation/event/effort | [43d](43d-dynamics-fitting-and-workflow-effectiveness.md); integrator and workflow owners | No interpolation across discontinuities; covered outputs can be served without new stepping; requested derivative products are atomic |
| Derived analysis and selected probe/capture | [43e](43e-diagnostic-analysis-artifacts-and-profiling.md); interpretation/math/artifact/capture owners | Hypotheses cite actual evidence; passive reads do no numerical work; diagnostic sufficiency is independent of science validity |
| Parallel lanes, waits and platform equivalence | [43f](43f-execution-resources-and-platform-profiles.md); existing execution/admission/native owners | Issued work stays owned through drain; conservative defaults persist until a concrete optional profile is selected |
| Assembled completion | [43g](43g-qualification-and-design-publication.md) | Focused controls, integration, scientific integrity and benefit claims establish different things |

These are meaning/operation contracts, not a mandate for a type, service or serialized instruction per concept. Rust-owned external documents and generated Python projections evolve together where required. Internal facts stay internal when no consumer needs a public schema. Accepted identity/wire changes use the existing decision/codegen route.

### Complete execution and truthful progress

Every complete execution can reference its request, preparation/products, actual attempts, effective choices, work facts, original conclusion and requested readiness. The governing qualification endpoint is absent when qualification never succeeds; terminal elapsed remains available. A response containing typed withheld covariance may be ready while covariance is unavailable. Failed attempts, postsolve work and publication/access tails remain explainable.

Native/original-oracle/transport producer scopes stay distinct. Counter generations identify actual populations, not alternately nested owners. Coupled numerical fields share a coherent event sequence and coordinate/norm convention. Process CPU/RSS, thread scopes, shared flights, server statement samples and operator work retain their scopes. Missing coverage is not zero, a stale interval is not a hang, and an inclusive duration is not an exclusive time partition.

Ordinary collection uses cheap local counters, coarse phases, prepared census, coherent checkpoints and bounded copied history/summaries. Detailed modes are explicit, finite and independently fallible. Off means optional collection absent; mandatory accounting/scientific qualification and baseline local bookkeeping remain. Hub Off, engine operator Off and zero retained history are different controls. No mandatory ordinary store writes, scalar-block timers, hidden failure probes or native-history vector that grows unbounded until terminal truncation.

### Efficient mathematical and native realization

Remove source-established incidental work without awaiting a full flight recorder. Product demand drives support/coordinate construction, weighted versus directional derivatives, compatible region optimization and cache projections. Existing packed trial seconds, narrowed axes, lower-order reuse within a group, implicit result reuse, symbolic LU and factor scratch are preserved. Changes avoid eager unrelated constraints/guards and preserve provider derivative/envelope obligations.

Use Symbolica/faer and fitting native libraries rather than project math/solver machinery. Context-owned reusable scratch, checked addend/coordinate maps and bulk publication remove copies while retaining complete output on failure. Stronger native state reuse is conditional on complete validity, not pattern equality or session retention. Numerical scaling never redefines physical budgets. Method profiles are explicit; conditional BTF, scalar brackets, Anderson, root presolve/FBBT, continuation and bound relaxation do not become universal defaults by assertion.

IDAS output service and native advancement change together: true boundaries govern hard stops, library interpolation serves eligible samples, roots/resets/scheduled changes preserve semantics, and cumulative effort remains honest across resets/replay. Fitting/shooting retains integration purpose and complete product upgrades.

### Useful analysis and exploration

Pure explanations distinguish more work, higher unit cost, displaced inner/postsolve work, contention/wait and changed outcome. Classifiers are named hypotheses with phase/sequence/coverage preconditions, not silent solver policy. Source mappings survive CSE, reduction, aggregation and dual recovery. Explicit probes have a selected point/operator, their own admission/budget and independently unavailable outcomes. Selected evidence can be unsuitable for a study without invalidating a scientifically valid result.

Keep Perfetto/perf/Samply and Arrow/DataFusion as the primary composable stack. Tool/provider/allocator/ISA/FMA/build explorations answer concrete questions. No mandatory install bundle, percentage gain,≤1% observer SLA,≥10 repetitions, per-commit performance gate, broad provenance proof or new fixed resource cap is adopted.

## Rule decisions

<a id="rule-decisions"></a>

The create-plan skill requires an explicit accept/reject for each review rule impact before its dependent target is fixed. **The operator explicitly accepted all eleven recommendations as written on 2026-10-17.** Acceptance selects the stated target or bounded investigation route, not product execution, a numerical winner or a broader policy relaxation. The retained-rule alternatives remain useful if later evidence reopens a decision; they are not the currently selected branch.

| Item | Recommended decision and consequence | Decision / date | Route and retained-rule alternative |
| --- | --- | --- | --- |
| RC01 | Accept explicit producer/coherent-checkpoint relationships and joined endpoint evidence, with versioned consumer migration where necessary | Accepted 2026-10-17 | Blueprint§23.1/typed boundary decision route before dependent migration; otherwise retain per-field evidence and do not claim joined iterations |
| RC02 | Accept an explicitly selected finite detailed stream/artifact contract with independent capture sufficiency; preserve ordinary passive/bounded operation | Accepted 2026-10-17 | Blueprint§23.1/new selected artifact contract; otherwise sampled bounded collection remains without full-trace promise |
| RC03 | Accept weighted/directional mathematical products and compatible region/projection semantics under the math owner | Accepted 2026-10-17 | Blueprint§14 and required short kernel/product-contract ADR; otherwise retain current selected per-output representation |
| RC04 | Accept a separately qualified optional derivative-arithmetic compiled route, with fallible providers outside it and conservative FP default | Accepted 2026-10-17 | ADR-0191 successor/extension and blueprint§14.5 as required; otherwise derivatives remain interpreted while other math corrections proceed |
| RC05 | Accept bounded investigation of stronger native reuse; select retained classification/scaling/numeric state only after concrete validity decision | Accepted 2026-10-17 | Blueprint§17–§18/native lifetime decision after C03 evidence; otherwise fresh numeric setup plus current allocation reuse |
| RC06 | Accept IDAS true-boundary stops with separately governed interpolation/advancement and cumulative effort | Accepted 2026-10-17 | Blueprint§18 dynamic contract route before D01 migration; otherwise retain sample stops and their explicit work limitation |
| RC07 | Accept declared conditioning/composition alternatives while retaining physical accuracy authority and current defaults pending family-specific evidence/decision | Accepted 2026-10-17 | Blueprint§16.1/strategy decisions where changed; otherwise current defaults and transformations remain, without tolerance/nominal rewriting |
| RC08 | Accept investigation/design of explicit optional platform numerical-equivalence profiles; retain conservative default and prohibit blanket relaxation | Accepted 2026-10-17 | Precision/ISA/FMA binding or ADR route only for a selected concrete changed profile; otherwise current flags and scoped claims remain |
| RC09 | Accept full technical eligibility of MKL AUTO/AOCL/HSL including product use if fitting, with concrete provider/ABI/equivalence decision before adoption | Accepted 2026-10-17 | ADR-0108 successor/provider route where required; otherwise current provider remains, with technical comparison distinguished from adoption |
| RC10 | Accept effect classification and separation of truly observational retention from mathematical identity only where established; version changed identity | Accepted 2026-10-17 | Current `Controls::identity`/blueprint§17–§18 and hashing/boundary route if affected; otherwise conservative invalidation remains documented |
| RC11 | Accept correction of blueprint§15 absence wording to acknowledge named findings/profiles, without claiming complete DiagnosticsToolbox parity | Accepted 2026-10-17 | Design publication/revision route; otherwise source capability is documented here without changing the architecture owner |

Routine dependency upgrades need no ADR solely because a pin changes. New native bindings/kernel contracts, D1–D14 changes, boundary/identity changes and accepted-decision replacements follow their actual decision rules. Accepted ADRs remain immutable. Concrete decisions discovered during investigations return through this table's owning decision explanation; acceptance of a route does not preapprove its unknown answer.

## Execution structure and prerequisite reasoning

Companions own their Work packages tables; this coordinator owns combined target/rules/disposition only. All package states start **open**, meaning proposed scope, not execution authorization. Source questions can be answered during detailed execution planning where cheap; unresolved decision-relevant probes remain bounded work packages.

Package IDs are scoped by companion, for example 43f/F01. In packet-route/prerequisite columns F01–F05 identify 43f packages; in Finding reference and review discussions F01–F08 identify review findings. Those are different identities/status owners, never interchangeable closure claims.

1. Obtain rule decisions; establish the narrow producer/availability/checkpoint foundations A01/A02 and native activation C01. A05 effect classification can run independently; changed identity waits for its conclusion.
2. Develop math B01–B05, native transport C02 and the combined IDAS D01 correction when their specific contracts are available. B04 can project current products; B05 can improve current implicit demand without region compilation. These are not a mandatory serial chain.
3. Settle native reuse/method profiles C03–C05 and dynamic alternatives D03 alongside the compatible corrections. Keep current defaults when evidence does not choose a replacement. Native extraction C06 consumes only the particular checkpoint/detail slice required.
4. Build interpretation E01, attribution E02, selected probes E03, retained views E04 and tool adapters E05 on their actual producer/mapping inputs. E01's existing activity explanations can migrate early; iteration classifiers require coherent adequate history. E04 needs A04 invocation joins. A full profiler stack is not a math/driver prerequisite.
5. Execution resources F01/F02 and platform investigations F03–F05 follow their contracts. B06 compilation and selected lane/platform adoption wait only for their concrete product/equivalence decisions, not unrelated tool installation.
6. G01–G05 assemble the selected target, qualify affected scientific/workflow journeys, complete static checks once functional scope is integrated, and publish enduring design through its proper route. Existing-plan assessment/updates remain excluded.

Logical independence does not grant parallel edit ownership. Assembly/execution/native callbacks, shared document schemas and workflow result code need coordinated editing. Immutable programs can be shared; mutable evaluator/provider/native scratch is exclusive. Outer study pools, evaluator lanes and native BLAS/factor teams consume the existing enclosing admission, including issued work through actual drain.

### Investigations and completion branches

| Review inquiry | Owner and bounded decision | Completion branch |
| --- | --- | --- |
| Q01 product lowering | B02/B03/B05/B06: compare fitting Symbolica/faer realizations under selected widths/support/providers | Qualified accepted correction/capability; unsupported shapes may retain explicit narrower realization but do not close remaining F05/F06 or accepted B06 delivery; failed target premise returns for decision |
| Q02 native reuse | C03: identify which pattern/classification/scaling/numeric state survives each input change | Implement established compatible reuse or retain fresh state with explicit incompatibility/rebuild facts |
| Q03 method profiles | C04/C05/D03: compare admitted methods/compositions under equivalent original obligations | Concrete family-specific selection or retained defaults; no automatic size/depth/μ/nominal winner |
| Q04 detailed collection | A03/C06: identify requested event family, completeness and owner-safe pinned hook | Implement finite supported route, or explicit unsupported/unknown with bounded sampled alternative |
| Q05 sparse diagnostics | E03 with B/C math/factor owners: define required operator/scaling/residual accuracy and library fit | Qualified selected partial-SVD route or retained bounded dense/withheld product with supported reason |
| Q06 platform equivalence | F03/F04/F05: define/verify required equivalence and actual execution scope | Select a concrete optional profile or retain conservative default; no silent reproducibility relaxation |
| Q07 tool value | E05 with F03: select capture that discriminates a real question and verify feasibility | Integrate useful owned adapter or document supported no-adoption/unavailable route; no install sweep |

A negative investigation result closes its question only when the evidence actually settles it and dependent scope is adapted. It does not resolve a source-established F finding whose correction remains necessary. Source inspection can settle needless work without elapsed measurement; quantitative improvements remain unclaimed until measured under named conditions.

## Finding dispositions

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| F01: per-invocation work/timing join | S01/S04/S05 | open | A04; D04 shared availability correction | **Interface-checked, production audit 2026-10-17:** joined invocation/endpoints exist, but native derivative success/retained shooting data can still be advertised as available scientific products after authored refusal. [D04](43d-dynamics-fitting-and-workflow-effectiveness.md#d04) owns correction; [A04](43a-operation-evidence-and-coherent-live-diagnostics.md#a04) consumes it. Earlier joined-population acceptance remains in the 43a Outcome. |
| F02: callback producer collision | S02/S03 | resolved | A01; C02 | Actual same-demand nested path preserves separate generations/populations and one charging owner; **Tested:** [43a Outcome](43a-operation-evidence-and-coherent-live-diagnostics.md#outcome). |
| F03: incoherent field history | S06 | resolved | A02/A03; C01/C06/E01 | Coherent rows, explicit sequence loss and current progress under history-off; **Tested:** [43a Outcome](43a-operation-evidence-and-coherent-live-diagnostics.md#outcome). |
| F04: disabled POUNCE timers as zero | S03/S06 | resolved | C01 with A01 | Disabled/enabled/running/interrupted/absent facts remain distinct; **Tested:** [43a Outcome](43a-operation-evidence-and-coherent-live-diagnostics.md#outcome). |
| F05: local derivative amplification | S02/S07 | resolved | B01/B02/B03/B04; selected B06 | Required support/products with preserved aliases/providers/guards and removed incidental construction; **Tested, 2026-10-17:** [43g Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) and owning companion numerical/product controls. |
| F06: broad implicit products/contraction | S02/S04 | open | B05 with C03/D02 | **Interface-checked, production audit 2026-10-17:** unknown-only Newton/qualified IFT and support contraction are implemented; the explicitly required extraction/RHS/contraction/derivative-verification/expansion stage facts remain combined. [B05](43b-mathematical-products-and-evaluator-efficiency.md#b05) owns the missing production diagnostic behavior. Earlier numerical acceptance remains in the 43g Outcome. |
| F07: callback/sparse materialization | S01/S03 | resolved | C02 with B01/F01 | Full mapped/bulk output, no partial publication, correct boundary validation and scope; **Tested, 2026-10-17:** [43g Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) and owning companion numerical/product controls. |
| F08: sample-constrained IDAS mesh | S04 | resolved | D01/D02/D04 | Genuine boundaries/interpolation, events/products and cumulative effort together qualified; **Tested, 2026-10-17:** [43g Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) and owning companion numerical/product controls. |

These IDs refer to the integrated review, not similarly numbered historical reviews. Local package completion never implies resolution of all contributors. Disposition changes require their full evidence and remain owned here.

## Opportunity and scenario routes

| Review opportunity | Work route and closure meaning |
| --- | --- |
| O01 native reuse | C03/B05; distinguish retained artifacts/scratch/pattern/numeric state/starts and record actual reuse/rebuild |
| O02 numerical compositions | C04/C05/D03; resolve bounded family choices, preserve original physics and native ownership |
| O03 math/compilation/lanes | B01–B06/C02/F01; no naive shared-worker parallelism or provider compilation |
| O04 native detail/refinement/diagnostic products | C01/C05/C06/A03/E03; exact hooks, finite collectors and selected numerical analysis |
| O05 platform/provider/allocator | F03–F05; explicit eligibility, equivalence and no unsupported gain |
| O06 tools/derived analysis | E01–E05/G03; owned optional capture/views and discriminating evidence |
| O07 notifications | F02; existing owners, no lost wakes or weakened native exclusion/drain |
| O08 observational identity | A05; classify actual effects before separating identity and migrate dependent consumers |

S01–S08 retain their [review definitions](../design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md#7-scenarios-numerical-stages-and-verification). G02 selects complete journeys from them; companions add adversarial cases without restating scientific authority. New model/property fixtures remain independent from the implementation being checked.

## External recommendation routing

The review crosswalk remains the source of corrected positions for all 44 R1 groups,43 R2 numbered recommendations and 29 U groups. The tables below assign each to this series; they are routing, not a second assessment or status ledger. “Investigate” retains the original conditional nature; “omit” retains a rejected subproposal's rationale. No item becomes mandatory adoption merely by appearing here.

### R1 section groups

| External reference | Package / decision route | Scope treatment |
| --- | --- | --- |
| R1-01 | A04/E01/E04 | Extend existing authorities and joined views |
| R1-02 | A04/D04 | Two endpoints plus terminal/withheld product distinctions |
| R1-03 | A01/A04 | Layered producer/attempt/source/charge identities |
| R1-04 | A01/A02 | Metric meaning, coordinates, coverage and generations |
| R1-05 | A04/G03 | Preserve charge dedup and unknown totals |
| R1-06 | A04/E01 | Non-additive elapsed and actual relationships |
| R1-07 | A04/E04/G03 | Individual records versus Criterion populations |
| R1-08 | B01/A04 | Selected derivative/capacity census and actual work |
| R1-09 | B01/B03 | Constructed/consumed operands, no universal cost ratio |
| R1-10 | B01 | Support-driven contributions with aliases/multiplicity |
| R1-11 | B03/B04 | Fitting product/projection and representation decision |
| R1-12 | B05/A04 | Implicit stages, existing factor/scratch reuse preserved |
| R1-13 | C04/B05/G03 | Full/reduced complete work and original outcome |
| R1-14 | D01/G03 | Controlled output-grid correctness/work observation |
| R1-15 | D01 | Event, interpolation and cumulative effort combined |
| R1-16 | A01/C02/G03 | Callback gather/oracle/publication costs and owners |
| R1-17 | A01/A05/G03 | Off/ordinary/detail and prepared hot facts |
| R1-18 | D02/D04/A04 | Actual integration/product/assessment purposes |
| R1-19 | D02/D03 | Recording/replay/backward generations and checkpoint state |
| R1-20 | B06/F05/G03 | Eligible arithmetic and actual compilation lifetime |
| R1-21 | C03/B04/B05 | Distinct reuse/eviction/incompatibility products |
| R1-22 | C01/C06 | Existing Ipopt timings; selected private detail |
| R1-23 | C01/C06/A03 | Correct POUNCE activation and finite history |
| R1-24 | C06 | Actual KINSOL/KLU owner hook or explicit unknown |
| R1-25 | C06 | Pinned PETSc typed logging/event/KSP/PC scope |
| R1-26 | C06/D02 | Dynamic factors and bounded Uno structured detail |
| R1-27 | C06/E05 | Selected actual profiling build, not inferred factor counts |
| R1-28 | C04/C05/G03 | Eligible smooth-NLP profiles and original acceptance |
| R1-29 | C04/C03 | Structure/full/reduced/actual Schur complete cost |
| R1-30 | C04 | Root/Krylov/recycle library-owned methods |
| R1-31 | D03 | Dynamic/derivative/checkpoint family decisions |
| R1-32 | C01/C04/D03 | Requested versus actual eligibility/fallback |
| R1-33 | C03/A04 | Allocation, cold state and permitted seeds distinct |
| R1-34 | C05/D03 | Equivalent original tolerance/scaling contract |
| R1-35 | E03 | Selected costly numerical probes |
| R1-36 | E04/G03 | Optional eligible-family profiles retaining failures |
| R1-37 | E04/E05 | Trace Processor SQL, census and linked columnar views |
| R1-38 | E05/F05 | CPU/heap tools, symbols and interpretation limits |
| R1-39 | E05/C06/F05 | All named tools considered through bounded capability routes |
| R1-40 | E05 | Existing inspect/capture plus short-run launch coverage |
| R1-41 | A02/A03/A05 | Distinct collection modes, local aggregation and overhead |
| R1-42 | A02/A03 | Coherent iteration records versus coalesced points |
| R1-43 | A04/E01/E05 | Shared process/server/client scope |
| R1-44 | Coordinator/G01/G03 | Source-directed priority and corpus, no global recorder gate |

### R2 numbered recommendations

| External reference | Package / decision route | Scope treatment |
| --- | --- | --- |
| Y1 | F05/G03 | Disclosed validated dev evidence; fitting existing optimized profile |
| Y2 | F03/F05 | Conditional O3/LTO/codegen with actual equivalence |
| Y3 | A01/A05/G03 | Prepared facts, batch policy and nested producer correction |
| Y4 | C01/C06/A03 | Availability first; optional bounded activation/history |
| Y5 | F03/G03 | Actual host placement/context, no portable CCD winner |
| L0 | A01/A04 | Scoped phase totals/distributions/CPU, no fake partition |
| L1 | A04/C01/D04 | Effective attempts, work and governing outcomes |
| L2 | A02/C01/C06/D02 | Coherent method-specific current/detail records |
| L3 | C01/C06/C03 | Actual factor/layout/order/scaling and unknowns |
| L4 | E02/E03 | Cheap supplied attribution versus selected extra analysis |
| L5 | B01/A04 | Prepared census and actual demand/cache/provider work |
| L6 | E05 | Selected counters/off-CPU/heap with attribution |
| N1 | C04 | Conditional BTF/scalar/root composition; current default retained |
| N2 | B03/B05/D02 | Weighted Hessian versus action; exact provider/scaling chain |
| N3 | C03 | Native/conditioning reuse validity, not a flag-only change |
| N4 | C04/C06 | Typed backend/order controls and compatible preparation |
| N5 | C04/C05 | Method-specific setup/μ/accuracy/bound policies; omit raw tol alignment |
| N6 | C04 | Conditioning layer independent of physical budgets |
| N7 | C04/E01 | Declared recycle methods and qualified raw-map explanations |
| N8 | C04/C02 | Eligible root presolve and complete recovery/witness |
| N9 | D01/D03 | Combined interpolation/effort; separate corrector/linear choice |
| N10 | C05/C06 | Explicit refinement composition and correct precheck meaning |
| N11 | C04/C05 | Continuation/FBBT decision; preserve dimensionless RR roles |
| E1 | B01/E04/E05 | Prepared work census and selected kernel replay |
| E2 | B02/E02 | Library joint CSE with all occurrence/independent meanings |
| E3 | B02 | Bounded compatible regions and preserved barriers |
| E4 | B04 | Lazy valid projections; existing within-group reuse preserved |
| E5 | F01/B02 | Exclusive lanes and fitting library batching/SIMD |
| E6 | B06/F05 | Qualified optional derivative arithmetic compilation |
| E7 | B01 | Narrow offsets/poison/retained-vector work |
| E8 | B05/C03 | Narrow Newton/IFT; no invalid factor transfer/check deletion |
| E9 | C02 | Prepared scratch/maps and atomic native output |
| E10 | C02/F01 | Construction validation/witness/bulk publication/pool scope |
| E11 | C06/C03 | Native-library copies/workspace/debug/pattern/arena ownership |
| E12 | F02 | Notifications with existing safety and drain |
| E13 | F04 | Actual-scope optional allocator comparison |
| D1 | F03/F04 | Provider eligibility and concrete ABI/equivalence choice |
| D2 | F03/F05 | Explicit optional ISA/FMA profile, conservative default |
| D3 | F03 | Declared equivalence scope, no blanket reproducibility relaxation |
| D4 | C03 | Stronger complete native compatibility and retained scaling |
| D5 | F01/F03 | Pool composition, narrow determinism and latency/throughput |
| D6 | C05 | Selected relaxation/projected original assessment |
| D7 | A05/G03 | Observer-cost visibility, omit universal≤1% gate |

### Unnumbered and cross-report groups

| External reference | Package / decision route | Scope treatment |
| --- | --- | --- |
| U01 | B01/A04/C01/D04 | Pipeline census/fallback/postsolve distinctions |
| U02 | A02/A03/E04 | Finite record implementation; no end-only truncation/store dependency |
| U03 | A02/A05/C01 | Cheap ordinary facts versus selected deep work |
| U04 | E01 | Explicit analytical hypotheses and adequate evidence |
| U05 | E02/E03 | Selected failure follow-up; no automatic new science |
| U06 | E05/F03 | Needed perf capability, not blanket sysctls |
| U07 | E05/F05 | Scoped native/Rust unwind and sampling decision |
| U08 | E05 | Actual host uProf/IBS capability or supported no-adoption |
| U09 | E05 | Question-driven PMU/branch/c2c/off-CPU/BCC |
| U10 | E05/A03 | Optional Tracy with caller subscriber composition |
| U11 | E05/G03 | Optional causal intervention with meaningful progress |
| U12 | F05 | Conditional trained PGO/BOLT realization |
| U13 | E05/G03 | Isolated instruction/simulated-cache scope, no noiseless gate |
| U14 | E05 | Heaptrack scope/coverage/perturbation |
| U15 | E05/F04 | perf-event2/dhat and distinct allocator scopes |
| U16 | E05/E04 | Comparable differential sample rendering |
| U17 | E05/C06/F05 | Tokio/native profiler/LLVM-specific questions |
| U18 | F04 | AOCL coherent provider/ABI/equivalence route |
| U19 | E03/B05/C03 | Library partial-SVD fit and correct operator/factor premises |
| U20 | C06 | Exact newer SUNDIALS capability, no Newton-hook inference |
| U21 | F04 | Full HSL technical eligibility; concrete adoption decision |
| U22 | C06/E05 | Actual six-option Ipopt logging/checking source route and effects |
| U23 | A04/G03 | Comparable populations; omit mandatory repeats/alerts/noise claims |
| U24 | E04/G03 | Optional eligible-family statistical/profile views |
| U25 | G02/G03 | Decision-relevant representative stress cases, no scale promise |
| U26 | A04/E04 | One immutable linked comparison product; selected bounded studies |
| U27 | Coordinator/G01 | Real prerequisites, all omitted-roadmap items retained |
| U28 | C01/G05 | Verified native activation/catalogue wording correction |
| U29 | Coordinator/G03/G05 | Source maturity and honest implementation/measurement claims |

### Explicitly omitted or constrained subproposals

The corrected positions in the review remain binding to this plan's target. Do not restore unconditional BTF/Anderson/size-based ordering defaults; raw tolerance alignment; nominal multiplication of dimensionless RR smoothing; eager unrelated-demand caching; first-only source attribution; unchecked original witness/factor transfer; universal pass/allocation/gain claims; native history called complete after decimation; always-on expensive factor/condition/top-k probes; a default bespoke Lanczos/AD/solver controller; blanket host privilege or all-tools installs; mandatory repeated benchmarks/per-commit timing gates; or environment/provenance proof. Their parent rows above route the useful obligation or bounded investigation, not the rejected mechanism.

## Verification and acceptance

**Tested, 2026-10-17:** the companion Outcomes and 43g record selected local Linux/pinned-toolchain/force-validation mechanism, scientific and consumer acceptance, with exact commands and exclusions. The following approach governed execution: each companion gives behavior, independent checks, affected consumers and required retirement. Compile touched closures and run targeted unit tests while implementing. Pure observation/interpretation/policy/evaluator checks use their actual dependencies and do not acquire unrelated store/native startup. Generate changed registries/contracts through their generators, never by editing outputs.

After functional scope is integrated, G02/G03/G04 select relevant native/component/integration/Python/scientific journeys, one complete diagnostic observation where useful, hygiene/manual native and feature checks, and documentation checks. Full Clippy continues after failures in selected ordinary/validated/native modes; every actual Arrow-consuming correctness closure explicitly activates force-validation. Fix all scope-owned failures against zero. A local pass is not assembled acceptance, and source-founded improvement is not a measured percentage gain.

Preserve original residual/bound/domain/closure qualification, derivatives/provider order, event/reset semantics, warm-start/result permissions, loss/unavailable/partial facts and actual cancellation/drain. Study evidence sufficiency never changes scientific permission. No mandatory repeated comparison or environment/provenance attestation is scheduled.

Overall completion requires each F finding's full correction, each selected product/consumer and deletion, all Q questions settled with their dependent branch, required rule/design publication, and G-qualified combined acceptance. Optional candidates may close by supported no-adoption; known findings may not. Current architecture and ADR rationale receive enduring meaning through the decision route; old-plan reassessment/updates remain outside this series' authorized document-creation scope.

## Current checkpoint

**Interface-checked — production-completeness audit, 2026-10-17:** the integrated production code at `d3283d3d1` was inspected against all 36 packages, F01–F08, accepted RC01–RC11, Q01–Q07, opportunity/scenario routes and the corrected external-recommendation routing. The operator clarified that production alignment, rather than validation records, is the audit's focus. No missing receipt, timing measurement or fresh qualification run is treated as remaining implementation work. No product code was changed during the audit.

The series is **in progress** at integration acceptance. All eight production corrections are implemented: prepared ordinary callback addresses; actual IFT stage observations; same-point shooting adjoint purpose; governing per-step scientific availability; one diagnostic operation clock; typed original invocation/attempt/failure context; owned launch CPU capture; and notification-based native/compiler waiting. Existing owners A04, B01/B05, D02/D04, E03/E05 and F02 retain their package status until the affected tests and scope-end checks finish. G01/G05 own integration and final publication. F01/F06 remain open until that closure.

One root checkout integrates the three bounded executor worktrees. Independent source review found and corrected term-analysis deadline propagation, whole-run authored-refusal origin lookup and final native-export deadline checks; typed stop causes remain intact. Generated source contracts are regenerated. Targeted native compilation/tests, the editable dev Python extension and scope-end checks are in progress; earlier acceptance remains scoped to its original source. No numerical accuracy, Q01–Q07 choice, existing-plan scope, formal ADR status or environment/provenance proof policy changes.

## Outcome (recorded after implementation)

### What was built

**Implemented — delivered scope, with remaining production work owned by the current checkpoint:** the standalone A–G target integrates coherent producer/current/history and invocation evidence; support-driven selected mathematical products, compatible optimization and optional scalar derivative compilation; owner-safe native transport, valid reuse and declared numerical profiles; true-boundary dynamics and complete fitting products; evidence-linked analysis, source attribution, selected singular modes and immutable query views; bounded profiling adapters; and notification-based resource waiting with exclusive admitted workers. Q01–Q07 have concrete qualified or supported retained branches, recorded in the companion Outcomes. The production-completeness audit reopens F01/F06 and the affected package deliveries; six original findings remain resolved. Previous qualification of the implemented paths is retained without asserting complete target implementation. The enduring contracts are published in the architecture and simulation diagnostics guide; ADR-0192–0196 remain formally proposed.

**Tested:** the [43g Outcome](43g-qualification-and-design-publication.md#outcome-recorded-after-implementation) owns commands, local Linux/pinned-toolchain/force-validation conditions, assembled original numerical expectations, scoped static acceptance, initial failures, successful affected reruns and exclusions. The [43a Outcome](43a-operation-evidence-and-coherent-live-diagnostics.md#outcome) retains its earlier foundation acceptance. Composite compiler and Python acceptance is identified as such. Controlled work counts establish removal of unnecessary work under unchanged obligations; no representative elapsed-speedup or universal whole-product qualification is claimed.

### A mistake made and corrected

Integration exposed nullability lost at the registry constructor, native empty-Hessian and restored-producer defects, canonical IFT axis admission, an existing Python property displaced by a new endpoint, a threaded pre-exec launcher risk and an Uno-only feature-gating mismatch. Each correction went to its actual owner with revealing numerical, contract, lifecycle or dependency-locality controls. Scientific assertions were retained rather than weakened to close findings.

### Deviations from the plan, deliberate

The operator delegated remaining decisions and requested design-phase proportion. Supported current numerical/provider/build/tool branches complete their bounded investigations without mandatory tournaments, repeated timing, global host tuning, a second scheduler or environment/provenance proof. Full bounded SVD serves the requested full rank/nullspace product; no native-factor handoff or all-platform performance winner is inferred. Original external reports and source review/evidence remain unchanged. Other plans retain their independent scopes/statuses; this series does not reassess or close them. The highest-numbered coordinator and companions consumed by proposed ADR verification and current qualification remain retained under ADR-0096.
