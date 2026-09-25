---
title: M22 local qualification review
date: 2026-09-24
status: reviewed
decision: Accept
reviewer: /root/final_claims_review
implementation-authors: [/root]
---

# M22 local qualification review

## 1. Scope and coverage

**Tier: design. Purpose: conformance.** Review the implemented Plan 14 target and
its M22 local Linux qualification under ADR-0087, blueprint §0.5, core standard
2.0 (DP-01–DP-24, G1–G8), and process-simulator profile 1.0 (PS-01–PS-13,
PS-G1–PS-G3). The implementation author is `/root`; the independent claims and
library reviewer is `/root/final_claims_review`.

This review owns G7/G8 and the final claims/authority reconciliation. **Accept the
assigned G7/G8 scope, with zero open MUST findings.** The independent
[runtime review](design_review_m22-runtime_2026-09-24.md) and
[scientific review](design_review_m22-scientific_2026-09-24.md) own their gate
decisions; their source arguments have been read, rather than inferred from agent
summaries. This is a bounded conformance review, not a second whole-system review.

The completed functional evidence is
`build/plan14/m22-functional-cost-input/checks.json`: 40 required checks and Q01–Q17
pass. `build/plan14/m22-costs/plan14-measure.json` records all 23 cost cases. Its
source digest is
`43a97300e00d83aa7e456f9b5a05fc65a4aaac2455833afe6e303892f3589aaf`.
The performance campaign deliberately stopped after measurements: its pending
review-collection step does not invalidate the measurements, and this review does
not relabel that checkpoint as completed Q18. The final collector binds these
independent decisions after documentation-only reconciliation.

Analysis modes are square simulation, NLP, coefficient LP/MILP/convex QP, explicit
cones, selected fixed-mass ODE/index-1 dynamics, and steady/transient fitting.
Journeys include edit/re-solve, structural edits, new specializations, recycle,
flash, vessel, fitting, publication/reopen, and in-flight cancellation. The review
does not extend support to general implicit/higher-index DAE, hybrid gradients,
global identifiability, covariance, JIT/SIMD, or a complete property catalog.

The maintainer-selected boundary excludes release-profile qualification, coverage,
exhaustive feature matrices, remote CI, other platforms, distribution artifacts,
and repair of the known upstream `proc-macro-error2` future-compatibility warning.
That warning remains disclosed; a local design-stage pass is not a claim of zero
warnings in every dependency or a release qualification.

The subsequent explicit maintainer decision, recorded in ADR-0087 and the M22
packet, also defers strict workspace Clippy cleanup. The failed Clippy logs are
retained evidence; this review cannot establish or claim lint-clean status. The
required workspace test already includes the governance crate, so removing its
duplicate standalone invocation does not remove those governance tests.

**Coverage method:** read current source, authorities, receipt continuation logic,
regression tests, exact functional witnesses, raw cost artifacts and companion
reviews; use lexical `rg` searches and pinned `ast-grep 0.45.3` structural matches.
Independently verified ten functional ancestor hashes, 230 case-artifact hashes and
all twelve measured executable/linked-library hashes. No new test or benchmark was
started by this reviewer. Search silence was not treated as semantic absence.
Workspace changes from other agents were preserved.

## 2. Authority map

| Meaning | Authority | Derived consumer and observed boundary |
|---|---|---|
| Current architecture and supported native scope | blueprint §0.5; ADR-0082–0084 | Current library integrations and public workflows; earlier displaced blueprint catalogs are expressly historical |
| Local completion boundary | ADR-0087; M22 execution packet | `scripts/validation_scope.py:122` selects local functional and performance gates; exclusions are explicit |
| Exact acceptance witnesses and cost shapes | `docs/plans/14-acceptance-cases.toml` | `scripts/plan14_acceptance.py:26` and `scripts/validation_cases.py:153` require exact identities, not aggregate counts |
| Dependency versions/features | Workspace `Cargo.toml`, resolved `Cargo.lock`, manifest profiles | Library calls and native provenance; no independent adapter version pin establishes authority |
| What actually executed | Origin campaign logs, JUnit, source/native manifests and raw Criterion files | Retained records preserve origin and authenticated artifact digests |
| Independent gate decision | Authored review argument | Review collector records evidence; it does not decide alignment or grant authority |

Binding K1 is resolved for this scope by following Plan 14 D02–D04 and blueprint
§0.5 over displaced custom-math/Pyomo text. K2 retains existing generated contracts;
no new generic generator is proposed. K3 maps historical DM identifiers through
core §I; accepted arguments need no terminology rewrite.

| Quantity/model element | Dimension and unit | Basis | Reference/convention | Envelope | Authority |
|---|---|---|---|---|---|
| Selected FeOS state | Temperature K; explicit density; pressure Pa | Molar methane/ethane/propane composition | Declared caloric convention and component order | Typed state envelope, including independently evaluated pressure | Provider ports/package and blueprint §0.5; `feos.rs:90`, `feos.rs:342` |
| Selected property outputs | Enthalpy J/mol; entropy J/(mol·K); log fugacity coefficient dimensionless | Molar; explicit phase | FeOS PC-SAFT/DIPPR package identity | Rejected outside declared operating bounds; empirical accuracy is not implied | `crates/pse-kernels/src/feos.rs:356` |
| Vessel and fit outputs | Physical source declarations and generated result relations | Declared amount/energy contributions | Same model and native workflow | Fixed diag(I,0) profile; smooth transient sensitivities only | blueprint §0.5; ADR-0084; scientific companion review must settle detailed admission and closure |

## 3. Contracts and enforcement

| Contract | Enforcement | Failure behavior | Evidence |
|---|---|---|---|
| A selected backend can consume the admitted mathematical class | `routing::select`, `crates/pse-backend-native/src/routing.rs:19` | Unsupported root/discrete/conic representation or unavailable backend returns an explicit error and alternatives; no fallback executes | **Implemented** |
| Discovery and execution refer to the complete declared witness set | `verify_selection`, `scripts/plan14_acceptance.py:37`; `validate_artifact`, `scripts/validation_cases.py:153` | Missing, duplicate, extra or wrong-profile evidence fails | **Implemented** |
| Native evidence refers to actual executable and linked-library bytes | `native_provenance`, `scripts/plan14_acceptance.py:95`; `verify_native`, `scripts/validation_receipts.py:40` | Missing linkage, changed binary/library or changed nested-thread budget fails | **Implemented** |
| A measurement checkpoint is not completed qualification | `scripts/validation.py` performance checkpoint/continuation; `scripts/validation_receipts.py:185` | Reviews remain pending; executable changes cannot retain measurements; any source change requires fresh reviews | **Implemented** |
| Review evidence is independent and current | `scripts/plan14_review.py:31` | Rejects wrong source, absent reviewer/implementation author, missing evidence, nonacceptance or nonzero open MUST findings | **Implemented**; author identity remains an accountable review assertion, not cryptographic proof of independence |

The reviewed functional-continuation change in
`scripts/validation_receipts.py:226` permits explicit impact selection for successful
observations collected during formatting changes. Fresh or newly observed drift
requires a reason and affected reruns. Retained checks keep their original
`changed_source`, `origin` and artifact hashes; selected checks rerun. Re-consuming an
already reviewed observation authenticates the ancestor chain without requiring a
new impact decision. Measurement/review observations with in-run drift cannot be
retained; executable changes invalidate measurements, and source changes invalidate
reviews. The tip's `source_unchanged` describes that continuation, not fresh
execution of every retained observation against the tip's bytes.

**Well-posedness:** class routing distinguishes roots from optimization and directly
evaluated constant cases. Structural rank is deliberately not numerical rank:
`StructuralAnalysis` retains semantic rows/columns and pounce-presolve provenance.
The scientific companion owns the complete pre-solve and dynamic admission verdict.
Its repaired higher-index refusal witness is present and passed in the final
127-test native selection.

**Outcomes:** native termination, candidate, independent physical quality and local
response rank remain separate. `FitReport` has optional solve/quality/candidate/
response/rank fields (`crates/pse-runtime/src/workflow/fitting.rs:81`); it makes no
covariance or global-identifiability promise. Numerical/physical profile gates are
settled by their independent review and executed witnesses, not by library names.

## 4. Derivation and execution

| Stage | Mechanism and scope | Inputs/reuse | Effects and evidence limit |
|---|---|---|---|
| Algebra and derivative programs | Symbolica evaluator plus Numerica `HyperDual` vectorization | Typed expressions, parameter order, derivative shape, optimizer controls | `crates/pse-math/src/library.rs:60` bounds compilation and rejects complex output; library evaluation replaces custom arithmetic |
| Structural analysis | pounce-presolve Hopcroft–Karp, DM and BTF | Complete admitted incidence and semantic mapping | `incidence.rs:285`, `:303`, `:357`; checked outputs are structural evidence, not numerical nonsingularity |
| Native solving | Ipopt C, POUNCE, KINSOL, HiGHS and Clarabel | Explicit class, representation, options and attempt state | Libraries own iteration/factorization; project code maps contracts and outcomes |
| Dynamic integration | Diffsol BDF/BDF sensitivities with faer sparse LU | Admitted fixed-mass profile, initial conditions, tolerances, parameters/events | `dynamics/integrator.rs:434`; library owns integration, consistent initialization and output integration |
| Cost collection | Criterion, standard process APIs, native metrics and tracing | Manifest's 23 exact workloads; dev Cargo profile; per-workload fresh process | `scripts/plan14_measure.py:39` builds before timing; cases include joined execution/validation and attempt teardown |
| Receipt/review collection | Existing runner, JSON/JUnit libraries, digests | Source/native identities, exact cases, origin chain, authored gate decision | Evidence is authenticated to its declared scope; it does not infer design alignment |

| Numerical stage | Formulation/derivatives | Scaling/class | Outcome/post-check responsibility |
|---|---|---|---|
| Algebra/provider composition | Original guards precede library simplification; Symbolica/Numerica and FeOS/num-dual derivatives | Typed input/output quantities; native problem class remains explicit | Scientific review settles guard/derivative qualification; no new exactness claim is inferred from library names |
| Algebraic native solve | Backend-specific admitted representation | Root, NLP, coefficients or explicit cones; declared tolerances | Native status and independent original-model quality remain separate |
| Dynamics/fitting | Selected smooth fixed-mass profile; no hybrid-gradient or uncertainty claim | Diffsol/outer native NLP with declared sensitivity/rank limits | Complete/partial/failed outcomes and physical checks require executed witnesses |

## 5. Journeys examined

| Journey | G7/G8 evidence and limit |
|---|---|
| Value edit/re-solve | Warm benchmark retains runtime/revision owner, edits case values and prepares through the public path; no compiled Rust cost is timed |
| Structural edit/new specialization | Medium 8/9-block edits and new expression identities exercise changed preparation; no claim that all graph work is reusable |
| Flash, vessel, fit | Dedicated cold/warm/difficult-start or physical scenarios use the same native workflow; success and physical outcome assertions remain in the benchmark |
| Publication/reopen | Dedicated workload commits through existing exact publication and reopens the committed version |
| In-flight cancellation | Workload waits for observable progress, requests cancellation, joins the job and requires typed cancelled termination |
| Unsupported class/provider work | Routing refuses unsupported representations; FeOS checks its envelope and explicit output demands |
| Extension | Add physical/model declarations and genuine new provider binding; keep algebra, matching and solver internals library-owned |

These are **Implemented** paths with **Tested** exact functional witnesses and
**Measured** scoped costs below. A cost case includes its physical/status assertions;
timing alone does not establish numerical validity.

## 6. Gates

| Gate | Verdict | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | n.a. to assigned verdict | Runtime companion owns this gate; its argument has been read | Collect its independently authored final decision |
| G2 Semantic fidelity | n.a. to assigned verdict | Scientific companion owns this gate; its argument has been read | Collect its independently authored final decision |
| G3 Validity | n.a. to assigned verdict | Scientific companion owns this gate; its argument has been read | Collect its independently authored final decision |
| G4 Hidden behavior | n.a. to assigned verdict | Runtime companion owns this gate; its argument has been read | Collect its independently authored final decision |
| G5 Consistency/recovery | n.a. to assigned verdict | Runtime companion owns this gate; its argument has been read | Collect its independently authored final decision |
| G6 Transformation/reuse | n.a. to assigned verdict | Runtime companion owns this gate; its argument has been read | Collect its independently authored final decision |
| G7 Truthful capability claims | **Pass** | Exact functional witnesses, 23 authenticated cost cases, explicit unsupported profiles, origin-preserving continuation, disclosed failed +15% flash start and strict Clippy exclusion | Preserve the stated limits in final active documentation and digest-bound collection |
| G8 Library leverage | **Pass** | Actual library calls own generic math, structural and native numerical work; retained code maps domain contracts and evidence | No product correction required |
| PS-G1 Physical consistency | n.a. to assigned verdict | Scientific companion owns this gate | Collect its independently authored final decision |
| PS-G2 Well-posedness | n.a. to assigned verdict | Scientific companion owns this gate | Collect its independently authored final decision |
| PS-G3 Numerical integrity | n.a. to assigned verdict | Scientific companion owns this gate | Collect its independently authored final decision |

## 7. Findings and principle verdicts

| ID | Finding | Principles/gate | Evidence/gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| — | No open G7/G8 MUST-level finding within the inspected scope | DP-13–DP-16, DP-21–DP-22; G7/G8 | Sources, executed witnesses, authenticated measurements and scoped claims above | No mandatory product correction identified | None proposed | Final collector preserves original evidence and independently authored decisions |
| O01 | Broad warm/thread speedup is not established | DP-22; G7 | `warm-large-4` is 979.559 ms versus `cold-large-4` at 752.596 ms; variability is visible in the raw intervals | A blanket warm-reuse or parallel-scaling claim would overstate the data | Retain per-workload observations, without an invented speed threshold | No defect in the scoped reported claims |
| O02 | The selected off-reference flash start is +2% density, after +15% failed locally | PS-12, DP-22; G7 | Preserved failed process log and final benchmark's `value *= 1.02` | This supports that selected starting point, not arbitrary-start robustness or global infeasibility | Keep the failed case disclosed; do not change physical references/tolerances | Current M22 packet explicitly records both observations |

The remaining collector/documentation continuation is evidence assembly, not an
invented product defect. Historical milestone notes remain historical.

| Principles | Verdict in this focused review | Basis |
|---|---|---|
| DP-13, DP-14, DP-16 | Satisfied | Actual selected libraries own evaluator/AD, graph analysis, solver iteration and integration; bounded domain mapping and evidence collection explain retained project code |
| DP-15, DP-21 | Satisfied in assigned scope | Explicit class/envelope/native-byte/source admission, executed profile witnesses and authenticated retained origins |
| DP-22 | Satisfied in assigned scope | Measured costs, failures, exclusions and continuation origins are explicit; no blanket speedup, lint-clean or empirical claim |
| PS-09, PS-12 | Satisfied in assigned scope | Explicit routing and local result/diagnostic boundaries; general DAE, hybrid gradients and uncertainty not advertised |
| Other DP/PS principles | Outside assigned verdict depth | Runtime/scientific companion reviews own their applicable verdicts; their arguments were read and are not replaced here |

The material strengths are refusal before unsupported dispatch, authentic exact
witness accounting, preservation of source/native origin on continuation, and
direct library ownership. Removing these would permit misleading acceptance or a
second numerical authority.

## 8. Library-leverage ledger

| Capability | Project code | Library/built-in considered and used | Fit/gap | Recommendation |
|---|---|---|---|---|
| Symbolic evaluation and derivative arithmetic | `pse-math::library` | Symbolica 3.0.0/Numerica evaluator and dual vectorization | Project code bounds symbols/options, preserves guards and maps physical bindings; no project arithmetic instruction engine | Retain thin integration |
| Property state and derivatives | `pse-kernels::feos` | FeOS 0.10.1; num-dual 0.14.2 | Project owns declared component/reference/envelope semantics; library supplies state/property/AD routines | Retain selected profile, not a general property claim |
| Matching/DM/BTF | `pse-structural::incidence` | pounce-presolve 0.12.0 | Library performs algorithms; project validates complete scope and maps ordinals to semantic identities | Retain admission/mapping |
| NLP/root/linear/conic numerics | `pse-backend-native` | Direct Ipopt; POUNCE 0.12.0/FERAL; KINSOL; HiGHS; Clarabel | Explicit contracts, FFI containment and independent physical checking remain project responsibilities; generic iteration stays native | Retain class-specific routes |
| Dynamics/linear algebra | Dynamic adapter and fitting response composition | Diffsol 0.16.2; faer 0.24.4 | Library integrator and sparse LU, source-aware admitted profile and physical outputs | Retain scoped integration |
| Timing/statistics | Native-process benchmark and collector | Criterion, tracing and standard process/filesystem APIs | Project code defines physical workloads/provenance; no bespoke confidence-interval estimator or timing framework | Retain bounded instrumentation |
| Acceptance/review accounting | Existing validation scripts | pytest/nextest/JUnit/JSON/hash/process APIs | Exact current-source and independent-review meaning is repository-specific; standard libraries handle mechanics | Retain existing runner; do not add a design-alignment engine |

## 9. Alternatives

| Alternative | Meaning/extension locality | Bespoke burden | Correctness/operational risk | Cost evidence | Choice |
|---|---|---|---|---|---|
| Current inspected target | Typed source declarations, one selected library per numerical responsibility | Physical contracts, mappings, admission and workflow composition | Actual boundary qualification and explicit retained-impact decisions | 40 functional checks and 23 dev-profile cost cases | Retain |
| Reintroduce custom solver/evaluator or compatibility compiler | Re-expresses arithmetic and convergence meaning | A second math/iteration implementation and caller paths | Conflicting behavior and unsupported fallback risk | No demonstrated benefit | Reject |
| Library-owned alternative | Same as current target | Narrow adapters over adopted library APIs | Preserves domain checks outside generic algorithms | No performance claim merely from adoption | Selected |
| Simplest viable qualification | Existing exact manifest/runner, library measurements and independent written review | No new alignment checker | Honest incomplete states; failures remain visible | 23 declared case scenarios, no invented speed threshold | Selected; coincides with library-owned alternative |

## 10. Verification and evidence limits

| Claim/risk | Label | Exact command or evidence | Mode and zero baseline | Current result |
|---|---|---|---|---|
| Repository command surface | Interface-checked | `just --list` | Read-only; command failures 0 | Surface inspected |
| Structural call-site search | Interface-checked | `ast-grep --version`; `ast-grep run --lang rust --pattern 'ffi::KINSol($$$ARGS)' crates/pse-backend-native/src/kinsol.rs`; `ast-grep run --lang rust --pattern '$A.optimize_tnlp_without_presolve($$$ARGS)' crates/pse-backend-native/src/pounce.rs`; `ast-grep run --lang rust --pattern 'hopcroft_karp($$$ARGS)' crates/pse-structural/src/incidence.rs` | ast-grep 0.45.3; three successful exact syntax matches, 0 parser errors | Confirms call syntax, not resolved semantics or runtime success; source/imports read separately |
| Full local functional target | Tested | `build/plan14/m22-functional-cost-input/checks.json` and its authenticated continuation chain | Default workspace and selected native/Python profiles; correctness uses force validation; required failure baseline 0 | 40/40 required checks passed, Q01–Q17 complete; retained results retain their actual origin |
| Workspace and selected native execution | Tested | `just test --profile ci --success-output final --config-file build/plan14/m22-functional-current/test-nextest.toml`; `just plan14-native --profile ci --success-output final --config-file build/plan14/m22-functional-ready/plan14-native-nextest.toml` | Workspace `pse-relations/force-validate`; native manifest feature graph; test profile, nextest ci; baseline 0 | 1,697 and 127 passed respectively; zero failed/error/skipped/not-run required witnesses; workspace observation retained under explicit reviewed formatting impact |
| Python public and reader journeys | Tested | `just plan14-python build/plan14/m22-functional-tested`; `just assessment-python-unit build/plan14/m22-functional-qualified`; `just assessment-python-component build/plan14/m22-functional-pass`; `just assessment-python-integration build/plan14/m22-functional-pass` | Current linked native editable profile; exact fixture for component/integration; baseline 0 | 4, 116, 18 and 4 passed respectively, zero failed/error/skipped/not-run |
| Evidence controls and doctests | Tested | `just plan14-tools build/plan14/m22-functional-cost-input`; `.venv/bin/python -m unittest scripts.tests.test_validation`; `just doctest` | Manifest-selected tooling 6 tests; isolated continuation suite 28 tests; force-validation doctests; baseline 0 | All passed; prior direct unittest log read; doctest gate is separate from workspace nextest |
| Complete cost and memory evidence | Measured | `just architecture-acceptance build/plan14/m22-costs --phase performance --functional-from build/plan14/m22-functional-cost-input --stop-after plan14-measure`, invoking `just plan14-measure build/plan14/m22-costs` | Rust 1.98.1, x86_64 Linux; cached dev-profile setup, force-validation, 23 workloads, one native nested thread, ten flat samples each | 23/23 measured cases passed with finite intervals and authenticated artifacts; zero required case failures |
| Independent final gates | Implemented collector; authored G7/G8 acceptance | Final `just plan14-reviews <output>` collector | Eleven independently authored gates, final digest, 0 open MUST findings | This review supplies G7/G8 acceptance; collection follows documentation-only continuation |

Command paths above are rendered relative to the repository; receipt `command`
arrays preserve the exact absolute argv, while `origin` identifies the actual
execution directory. No retained command is claimed to have rerun at the final tip.
The original rejected strict Clippy invocations and the upstream warning remain
disclosed outside this revised local qualification scope.

The two exploratory ast-grep patterns for `optimize_tnlp`/`optimize` returned no
matches (exit 1); the actual `optimize_tnlp_without_presolve` call was then found
lexically and structurally. Those search results are not failures against the test
baseline and do not prove API absence.

**Measurement interpretation:** Cargo compilation occurs before measurement.
Cold means a fresh process-case/runtime owner inside an already-built application;
warm means a retained revision/runtime owner and value edits. Each workload has a
dedicated benchmark process, so VmHWM is a process-lifetime peak, distinct from
pool reservation observation. Phase averages include benchmark invocations/warmup and are
inclusive diagnostic timings; Criterion raw samples/intervals own total timing.
Unavailable conversion/property-state subclocks remain unavailable. There is no
JIT measurement, release-build performance claim, process-wide memory cap or
demonstrated blanket system speedup.

| Representative measured case | Mean ms | 95% interval ms | Interpretation |
|---|---:|---:|---|
| cold-small-1 / warm-small-1 | 176.965 / 48.003 | 174.927–179.228 / 47.689–48.275 | Retained-owner value edit is cheaper for this small case |
| cold-large-1 / warm-large-1 | 742.853 / 630.860 | 738.100–748.074 / 622.582–639.318 | Smaller benefit at the larger one-thread shape |
| cold-large-4 / warm-large-4 | 752.596 / 979.559 | 744.144–761.434 / 791.995–1193.960 | Warm/four-thread operation is not universally faster |
| flash-cold-1 / flash-warm-1 | 2579.687 / 1216.655 | 1760.275–3457.892 / 850.553–1759.718 | Broad workstation variability; selected physical flash only |
| flash-difficult-1 | 1010.728 | 960.199–1069.387 | Both initial densities are 1.02 times the reference, not arbitrary difficult starts |
| vessel-cold-1 / fit-cold-1 | 149.982 / 648.856 | 149.155–150.877 / 647.206–650.662 | Conserved dynamics and one-parameter transient fit |
| publication-cold-1 | 1854.987 | 1840.906–1869.588 | Includes native work and exact local publication/reopen |
| cancellation-cold-1 | 175.920 | 174.806–176.959 | Complete case admission/preparation/after-entry cancellation/join; not cancellation latency alone |

Across all cases, pool reservation peaks are approximately 16.42–21.32 GiB and
process-lifetime VmHWM approximately 157.4–225.5 MiB. These are different metrics:
the larger finite workstation allowance is conservative admission policy, not
eager allocation or an empirical RSS bound. All 23 rows, raw samples, phase
observations and intervals are retained in the measurement artifact and summarized
in the M22 packet.

The earlier +15% density benchmark failed its physical-acceptance assertion with
closure approximately −0.078419 against tolerance 1e−7. Its log remains at
`build/plan14/m22-measurements/process-cost/flash-difficult-1/process.log`.
The final +2% workload was selected afterward; the physical reference and tolerances
remain unchanged. This is explicitly scoped measurement, not evidence that the
earlier start converges, nor proof of global infeasibility.

Shared physical/model checks and independent thermo/derivative/conservation
oracles are covered by the scientific review and executed exact Q01–Q17 witnesses.
This review does not turn local comparisons into empirical property certification.

## 11. Authority changes and exceptions

No new product design or policy change is recommended. ADR-0087 and blueprint
§0.5 express the authorized local boundary. The concrete closure edits were
reviewed, including the active-document changes actually applied by the execution
owner; decision acceptance and final evidence collection are subsequent steps:

| Active description | Required reconciliation | Route |
|---|---|---|
| README, AGENTS, STATUS and plans index | Applied active text correctly limits completion to local Linux and retains strict Clippy/release/platform exclusions | Ordinary active documentation update |
| Plan 14 current state/Outcome, execution inventory, M22 packet | Applied text cites current campaigns, retained origins, failure baseline and measurement limits; historical checkpoints remain historical | Active plan/inventory update |
| Foundation and workflow | Applied text distinguishes tested local editable/native paths from distribution repair; native tear table reconciled after reviewer observation | Active contract/guide reconciliation |
| ADR-0082–0084 current pending-reconciliation sentences | Reviewed concrete proposed edits align local decision status with blueprint §0.5 and preserve predecessor arguments | Authorized local decision reconciliation under ADR-0087 |
| ADR-0085–0087, index/register | Reviewed concrete acceptance edits distinguish local acceptance from future PR/merge/release requirements; final status/index readback and documentation gates follow | ADR/index/register owners |

There is no SHOULD exception requested by this focused review. Excluded release,
platform and distribution claims are outside the supported closure scope. No
M22 functional failure may be excused by those exclusions.

## 12. Decision

**Decision: Accept for the assigned G7/G8 conformance scope. Zero open MUST
findings; zero required functional or final cost-case failures against baseline
zero.** Library ownership, actual capability witnesses and honest measurement
limits support this decision. It does not waive a failed scientific requirement,
accept strict Clippy findings for ordinary merge, or establish release/platform/
empirical coverage outside ADR-0087.

| Priority | Remaining closure work | Finding relation | Acceptance evidence |
|---|---|---|---|
| Correctness and authority | Complete reviewed local decision/status changes and collect all independent gate decisions | No open G7/G8 finding | 40 required functional checks and Q01–Q17 passed; final documentation-only continuation preserves origins |
| Library ownership/locality | Preserve inspected selected-library boundaries | G8 pass | Final digest includes the inspected paths |
| Measured cost | Retain all 23 raw workloads, intervals, memory metrics and the disclosed prior failure | G7 pass; O01/O02 delimit interpretation | Authenticated process-cost-v2; zero final required case failures; no broad speedup claim |

Reviews are evidence, not authority. This supplies the claims/library review
decision; the final source-bound collector and the other independent reviewers'
decisions complete the documented closure process.
