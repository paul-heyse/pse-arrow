---
title: Modeling kernel K4–K7 execution
status: done
date: 2026-09-26
parent: docs/plans/21-modeling-kernel.md
adrs: [ADR-0098, ADR-0099, ADR-0100, ADR-0101]
review_sources: [docs/design_review/reviews/design_review_modeling-kernel-k4-k7_2026-09-26.md]
---

# Modeling kernel K4–K7 execution

Authorized 2026-09-26. This packet owns execution progress; Plan 21 owns finding
dispositions. Implement Rust and Python operations, the complete generic diagnostics
catalogue and case-set studies. Specialized covariance, sensitivity, rolling-horizon
and training workflows, K8 science and K9 campaigns remain separate.

## Sequence and progress

| Step | Owner and responsibility | Targeted acceptance | State |
|---|---|---|---|
| 1 | Registry, parser and pure checked transformation/analysis declarations | Round trips, typing, policy/target refusals | implemented; targeted functional controls below |
| 2 | Compiler grouped projection into existing math assembly | Sharing, bindings, incremental equality, cancellation | implemented; targeted functional controls below |
| 3 | Generic shaped external functions and verified piecewise | Vector derivatives, lazy guards, continuity controls | implemented; targeted functional controls below |
| 4 | Continuous domains before finite expansion; derivatives and integrals | Polynomial exactness, discretization order, lineage | implemented; targeted functional controls below |
| 5 | Inline/nested/accelerated implicit stages; KINSOL and faer | Values/derivatives, regimes, typed failures | implemented; targeted functional controls below |
| 6 | Immutable elastic and continuation transformations | Signs, penalties, original-space recovery | implemented; targeted functional controls below |
| 7 | Cases, numerical sources/derived nominals and qualification | Precedence, hand scales, closure/checks | implemented; targeted functional controls below |
| 8 | Starts, stage variants and adaptive initialization | Dependencies, accepted transfer, rollback | implemented; targeted functional controls below |
| 9 | Steady, integrated and simultaneous dynamics | Generated residuals, ICs, events, partial outcomes | implemented; targeted functional controls below |
| 10 | Bounded generic diagnostics | Analytic examples, LP/MILP findings, inconclusive outcomes | implemented; targeted functional controls below |
| 11 | Case-set studies and Rust/Python operations | Reuse, failure isolation, warm dependencies, ownership | implemented; targeted functional controls below |
| 12 | Authored tests, shared checks and conformance command | Coverage, tolerances, oracle attribution, pure execution | implemented; targeted functional controls below |

## Binding decisions

The single compiler workspace calls pure modeling operations. Math owns Symbolica and
faer; backend-native owns native algorithms; runtime owns admission and effects. Durable
contracts are registry-generated. Group bodies feed existing CasePlan assembly.
Continuous domains resolve before finite enumeration. Original additive terms and
contribution magnitudes survive simplification for diagnostics and independent closure.

Nested stages use an injected inner solver on the outer admitted worker, without
recursive CPU admission. KINSOL sign constraints do not encode boxes: intervals are
trial guards and final acceptance checks. Verify residuals and bounds; compute implicit
derivatives using faer solves, never an inverse. Regime ties, singular roots and unproved
transitions refuse unsupported smooth derivatives. Default starts are deterministic;
mutable warm state is attempt-local.

External functions declare shapes, physical meaning, implementation/data revisions,
derivative source/order and validity. Piecewise claims require symbolic boundary
agreement, including relevant mixed partials. Unknown is not proof.

Numerical precedence: Analysis, Case, Model, ModelHint, PropertyDefault, DerivedNominal,
QuantityNominal, CanonicalFallback. Stages/relaxations are immutable overlays. Diagnostics
cannot upgrade solve outcomes; failed local solves do not certify infeasibility.
Concrete definitions need authored fixture bindings before shared checks attach.
Oracle metadata is portable; production does not import IDAES or require skill files.

## Retirement and verification

Delete the whole-root executable projection after grouped callers move. Migrate external
contracts before deleting FeOS-shaped columns/factory dispatch; retain science until K8.
Delete replaced nonprimitive function handlers, dynamic state/RHS authoring and their
callers/fixtures once replaced. Keep independent oracles and storage/schema conformance.

The maintainer explicitly confirmed retention of live science consumers until K8.
`workflow::sources::factory`, the `native_providers` FeOS columns and their vessel/reaction
consumers remain together. `dynamic_cases` and the prepared-transient fitting route remain
together. K8 removes each set when its scientific replacement and all callers have moved;
K4–K7 introduces no compatibility adapter between those declarations and the source kernel.

During functional implementation, use targeted package compilation, selected units,
force-validation through recipes and code generation with registry changes. The maintainer's
replacement instructions put integrated and static qualification after the complete
functional scope. That final phase is now active. K8 seed science and K9 measurements retain
their separate acceptance. No K0–K3 result is evidence for this packet.

## Current checkpoint

The authorized through-K8 work corrected fixture execution, failure transport, scheme and
realization contracts under E2/E3 of the [K8 execution sequence](21-modeling-kernel-k8-execution.md).
That packet owns the completed assessment and consumer migration evidence. Earlier
evidence retains its original scope. K9 remains excluded.

The compiler now projects shared consumer bodies into the existing CasePlan owner;
the whole-root executable body has been removed. Continuous axes realize finite differences
and Symbolica Radau/Legendre elements before finite expansion, retaining coordinate lineage,
quadrature, derivative and continuity rows. Nested implicit providers use the existing
guarded evaluator, injected KINSOL iteration, original residual/interval checks and faer
implicit first/second derivatives. Polynomial acceleration uses Symbolica certified real
roots and requires one verified root in the selected interval.

Source declarations now include discretization, realization, elastic relaxation, physical
continuation endpoints and piecewise continuity claims. Elastic variants retain original
equations and positive typed nominal parameters; signed nonnegative slacks contribute
dimensionless L1 penalties. Named stages select equation replacements without changing
variable identities. Chained piecewise functions have symbolic value/first/mixed-second
boundary checks before lazy branch derivatives are admitted. Direct external first and
mixed-second partials compose with the outer derivative jets. Nonprimitive tan/log10
handlers have been replaced by authored math definitions; guarded sigmoid/softplus
remain finite at extreme inputs.

Generic external declarations now check physical shapes, output selectors, implementation/data
revisions and derivative source/order against supplied native capabilities. Value and derivative
composition is covered by a vector fixture; external science retirement remains separate work.
Pure-function partials now reify Symbolica's unknown-function derivatives as supplied external
partials. Symbolica owns the chain rule, including nested external calls; the scheduler only
binds the resulting local derivative requests. Targeted vector and nested-cubic tests cover
first and mixed-second values, remaining outer derivatives, original-domain rejection and
refusal after the supplied derivative order is exhausted. Unproved external branch boundaries
remain a typed refusal.
The generic provider descriptor no longer contains FeOS components, phase or validity-envelope
fields. FeOS owns those policies and includes them in its data identity. Its targeted property,
derivative, ordering, stability and envelope tests pass; the legacy FeOS source relation still
has active science consumers and is not yet retired.

Function declarations support argument-only `valid(...)` predicates. The same source typing,
purity, recursion and specialization rules cover their bodies and predicates. An opaque domain
region executes predicates at value order and exports only an effect token. It preserves
rejection through normalization and partial derivatives without treating a Boolean indicator
as a smooth numerical function. Targeted source/compiler/math tests cover lazy guards,
second derivatives, value-only external predicates, cancellation and output-specific demand.

Solver views exclude observation rows and preserve fixedness/bounds as immutable case overlays.
Compiled hint observations resolve starts in dependency order and feed the existing numerical
source resolver; derived row nominals use original terms at the frozen nominal point. Validity
intervals become guarded math obligations, surviving simplification. Post-solve checks,
original elastic residuals, closure and reports supplement the native outcome. Results and
observations retain runtime allocation owners.

Bounded stage/homotopy orchestration and case studies use accepted predecessor values only,
preserving the final specification on failure. Initialization retains preparation errors and
native outcomes in one ordered attempt history; every attempted step counts toward its limit.
The overall deadline covers preparation, solving and qualification, and interrupted work joins
before return. Native callback recovery evidence survives disabled event history and distinguishes
recoverable evaluation stops from terminal failures. Targeted tests cover adaptive nonlinear
steps, preparation and callback rejections, cancellation/deadline joining, dependency isolation
and original-specification restoration. Only a separate accepted original-model solve commits.

Implicit projections now retain semantic parent identity and capture child providers inside the
parent residual. Petgraph orders these dependencies and rejects cycles. Runtime registration
attaches only providers consumed by each body; child configuration identity reaches its parent.
A targeted native fixture checks a child solve inside a parent solve, original values and
analytic first/second derivatives through both levels on the admitted worker. Sibling-provider
hints now participate in dependency ordering, and nested observations can seed outer starts.
A targeted test covers the ordered workflow and cyclic refusal. The compiler selects common
and regime-local hints once and captures their inputs in the residual's lexical scope.
Attempt-local hint programs now evaluate starts, intervals and nominal magnitudes at each
enclosing trial; the same numerical policy resolver applies their precedence. Residual
programs and sparse symbolic factors stay compiled. A native fixture changes the child's
tight interval with the parent's unknown and checks analytic first/second derivatives.
Self-dependent hints refuse, while selected regime overrides replace common hints. The old
registration-time observation path and its global hint dependency scan have been removed.
Observation preparation now traverses only the requested providers and their dependencies,
using petgraph's reversed traversal and topological ordering. A targeted native test verifies
that an undemanded implicit block with no start does not block an independent observation.

Inner scaling declarations now retain a separate program of original additive terms.
The worker evaluates that program at resolved variable nominals and current enclosing
inputs, then applies the same derived-nominal precedence as outer rows. Selected regime
scales replace common scales; hidden inner rows are excluded from outer scaling. Targeted
checks cover hand-computed magnitudes, model overrides and native nested derivatives.
Inner solves consume the resolved physical variable nominals and acceptance budgets.
KINSOL receives positive reciprocal scaling vectors; configuration identities retain those
inputs. A targeted native test exercises a large-magnitude root that cannot be reached under
the same limits with unit variable scales. The compiled library support now supplies the
actual sparse unknown Jacobian to KINSOL and faer. A targeted two-unknown test covers sparse
root values, implicit derivatives and refusal of a structurally deficient square system.

Authored regimes now execute through compiler/runtime registration and the existing native
KINSOL capability. Source uses `implicit ... select minimum(score, tolerance)` with
`regime ... eligible(predicate)` alternatives over shared unknowns. Alternatives retain
independent equations, starts, bounds and numerical hints; compiled identity includes all
residuals, eligibility and score expressions. Source typing and refusal tests, mathematical
selector tests and a targeted native package workflow cover minimum selection and ties.
The selector verifies every root and refuses failed alternatives, cancellation and exhausted
time. Selector derivatives remain unavailable without a transition proof, even when the
individual residual systems are smooth.
Pure authored expectations now execute through the compiler/math owner without constructing
a runtime or solver. Authored fixture values use the common case-value resolver; explicit
values override them. Tests distinguish declaration and instance identities and cover
failed expectations, invalid coordinates, cancellation and recovery.

The integrated time route now builds the existing native dynamics contract from generated
rate and explicit lower-endpoint equations. A start remains a guess, not an initial condition.
Targeted Diffsol fixtures cover normalized physical outputs, parameter/IC sensitivities,
semi-explicit DAE consistency and singularity refusal, and per-trial bounds/validity guards.
The same authored definition is checked in steady KINSOL, native integrated and simultaneous
backward-difference modes. The shared dynamic evaluator retains its cache/cancellation test.
Coupled derivative equations now compile into one guarded residual proved affine in its
rate coordinates. The compiler retains all contributing source equations and uses the
existing implicit provider owner with faer sparse LU; symbolic factorization is reused,
coefficients are evaluated per trial, and original residuals and bounds qualify the rates.
Targeted tests cover a parameter-dependent coupled system and its trajectory sensitivities,
singular matrices, nonlinear-rate refusal, exact second implicit derivatives and cancellation.
Definite integrals now use native quadrature and produce terminal reports only after the
whole declared domain completes. Intermediate cumulative integrals remain native trajectory
data, not values of the authored definite integral. Terminal checks run at the endpoint;
ordinary checks retain their sample scope. Future-dependent RHS/initial/flux functions refuse
forward integration. A targeted fixture covers a constant integrand, a logarithm that would
be invalid at a fabricated zero integral, and source checks; shortened horizons and terminal
integral sensitivities explicitly refuse. Native quadrature identities are separate from
optional conservation claims and reuse the existing output-integration algorithms.
Source event bindings now select guard/reset expressions and Boolean mode facts from the
same definition. Modes compile before admission and require identical state, parameter,
output, quadrature and physical-coordinate layouts. Each mode owns its provider values,
bounds, validity and sampled assessments. Native samples retain the actual mode after
coincident resets. Targeted native checks cover state-triggered resets, moving-event
sensitivities, changed rate equations, wrong-unit refusal and terminal prefix outcomes.
Legacy dynamic consumer migration belongs to K8 under the confirmed retirement boundary.

Generated check/report tables preserve indexed target identities and physical units, retain
allocation owners beyond their source handles, and distinguish validity membership from
permission to extrapolate. Authored fixture specifications carry typed values, fixedness,
bounds and expected DoF; portable oracle metadata identifies source assertions, not an
upstream execution. Their targeted physical/round-trip test is green. The shared conformance
runner attaches preparation, source coverage, DoF, envelope, start-to-solve, closure and
expectation checks to authored tests and invokes POUNCE's first-derivative/sparsity checker.
Its targeted native fixture and uncovered-definition/owned-table checks are green. The checker explicitly rejects incomplete evidence after
callback failure, cancellation or an empty sample; its focused native unit is green.

Python now exposes initialization histories, case studies, bounded diagnostics and integrated
simulation through owned native handles. Diagnostic preparation resolves the common case and
numerical policies without native solver admission; Jacobian and term checks retain nested
provider registrations. Targeted native checks cover numerical versus structural rank,
underdetermined nested cases and HiGHS analysis. Python checks cover explicit diagnostic
profiles, coordinate-labelled singular modes and ownership after dropping source handles.
Studies retain declaration-preparation failures per point and continue independent points;
dependent points still require an accepted predecessor.

Coefficient diagnostics now use the existing HiGHS adapter for native IIS, rays, ranging and
explicit-penalty relaxation. Nonlinear models and undischarged coefficient domains refuse that
route. The native IIS request selects the pinned library's full reduction rather than its
light default and supplies the dedicated remaining IIS time allowance. Discrete models use
an explicit continuous copy with binary and semi-variable bounds preserved as a relaxation.
Targeted tests distinguish an LP contradiction from integer-only infeasibility and retain
original semantic row/column identities.

Diagnostic samples share one prepared model, aggregate finding/time/sample limits, isolate
evaluation failures and join interrupted work. A sample cannot change frozen coefficient
inputs; that requires a new case. Source gathers now support unused-variable and
inequality-only-variable findings plus fixedness, bounds and incidence counts. The portable
`packages/reference/diagnostics/idaes-2.13.json` profile supplies explicit threshold data;
matching its values does not claim identical IDAES warning sets. Python sample handles and
the profile's public boundary checks pass, including independent point failures and retained results.

Trajectories project physical outputs, sensitivities, events and actual termination into the
existing generated runtime relations with columnar allocation ownership. The targeted Diffsol
test checks normalized state versus physical output, initial sensitivities and retained tables.
Shared physical assessments now run over explicit requested samples, including algebraic
residual budgets, authored checks, validity and independent original-contribution closure.
Generated check rows retain sample ordinal and time; completed integration and accepted
physical checks remain separate. Native termination is unchanged when a model check fails.
Terminal integral reports have the same physical result transport. Targeted tests cover
acceptance, failed checks, extrapolation membership and ownership after source handles drop.
This evidence concerns requested samples, not continuous certification between samples.
The targeted native Python module now checks successful simulation, source events, sampled
checks, terminal reports and ownership after dropping source handles. Its physical time
contracts are authored fixture data. The source event/root work uses Diffsol's existing
engine; no new integration or root-search algorithm was introduced.

Nonlinear explanation now composes immutable equation omission and the existing elastic
transformation, with explicit physical row nominals. A bounded deletion pass retains each
original-space native result and selected row set. Feasible witnesses, qualified stationary
positive-slack outcomes and inconclusive attempts remain distinct. Candidate explanations
are local observations under unchanged bounds, fixedness and inner systems; they never
assert global infeasibility or a mathematically minimal infeasible set. The targeted Ipopt
fixture removes a redundant constraint, verifies original specification restoration, and
keeps iteration limits inconclusive. Shared deadline/cancellation work joins before return.
The owned Python explanation boundary also passes its focused native test, preserving
trial classifications and original-model checks after source handles are dropped.

Diagnostics, sampled diagnostics, initialization, studies and nonlinear explanations now
project into generated, owned runtime tables. Histories retain per-attempt failures and link
successful preparations to result IDs. Matrix vectors use semantic coordinates; trajectory
samples retain authored mode names. The targeted Python module checks table contents and
ownership after dropping source handles. Native linear and Jacobian-optimization transports
also preserve source coordinates, scaling, bounded attempts and evidence scope; their Rust
projection checks and the expanded Python boundary checks pass. Ordinary Jacobian reports
also retain source-keyed physical row and variable nominals. Finite, infinite and
indeterminate observations remain distinguishable without violating finite Arrow fields.

Structural preparation now requires the parameter and fixed-variable values consumed by
compilation, while free starts remain optional for diagnostics. Missing values stay missing
in the report; no guessed point is published. Targeted tests cover missing starts and
out-of-bound free candidates while numerical solve admission remains strict.

A targeted scaled HiGHS relaxation test exposed two native API details: the operation restores
the original model status, and its objective can retain the original constant. The adapter
now makes the diagnostic copy's minimization purpose and zero offset explicit, reports the
operation code and restored status separately, and recovers the weighted penalty to physical
coordinates. A restored `NotSet` status is not labelled a relaxed optimum. This correction is
covered by a native test with a nonzero original objective constant and nonunit scaling.

K4–K7 functional implementation is complete within the authorized generic-kernel boundary.
The diagnostics catalogue audit covers structural and physical admission, candidate bounds
and values, residuals, scaled Jacobian norms and parallelism, SVD, original terms, domain
obligations, HiGHS LP/MILP evidence, local nonlinear explanations, sampled cases and model
statistics. The conformance audit covers concrete fixture coverage, DoF, derivatives,
envelope rejection, start-to-solve, independent closure, expectations and oracle attribution.
Unsupported evidence remains explicitly inconclusive; it does not become a successful check.

Plan 21 retains decision acceptance and the separately proposed K9 campaigns.
ADRs 0098–0101 remain proposed; blueprint revision 59 records the authorized architecture
reconciliation. The [K8 packet](21-modeling-kernel-k8-execution.md) owns seed conformance,
retirement and the completed assessment, including its unsuccessful static checks.
This packet makes no zero-warning or clean comprehensive qualification claim.
K9 measurements and integrated campaigns remain excluded.

## Verification

**Tested:** local Linux, pinned Rust 1.98.1 and Python 3.14.7. The selected Rust units use
`pse-relations/force-validate`; Python uses the editable native-solvers extension built with
`force-validate`. Every selected final scope below has **zero failures against a zero baseline**.
These scopes test the kernel and its boundaries; they do not establish seed science,
IDAES numerical parity, integrated product acceptance or other-platform support.

| Command | Selected result and conditions |
|---|---|
| `just unit-package pse-compiler 'test(workspace::modeling::tests::)'` | 31 passed: grouped compilation, physical typing, derivatives, continuous lineage, collocation, implicit stages, elastic variants, piecewise proofs, external functions, cases and pure authored fixtures |
| `just unit-native-package pse-runtime pse-runtime/native-solvers 'test(workflow::modeling::)'` | 29 passed: KINSOL nesting and regimes, initialization, studies, Diffsol ODE/DAE/events/quadrature, checks, diagnostics, HiGHS and local Ipopt explanations, conformance and retained results |
| `just py-unit-native python/pse/tests/test_modeling_kernel.py` | 5 passed: typed Python settings, authored conformance command, owned tables, event/mode/terminal reports, diagnostics including missing/nonfinite points, local explanations, initialization and studies |
| `just unit-native-package pse-backend-native pse-backend-native/highs 'test(native_relaxation_preserves_)'` | 1 passed: maximizing original objective with nonzero constant, nonunit variable/row/objective scales, restored native status and physical relaxation penalty |
| `just unit-package pse-modeling 'test(kernel_continuous)'` | 4 passed during implementation: finite expansion, coordinate identity, policy/domain/budget refusals and annotation typing |
| `just unit-package pse-math 'test(collocation::tests)'` | 1 passed during implementation: polynomial differentiation and integration through the library stencil |

**Implemented:** schema changes were regenerated with `just codegen` and the scoped
`just codegen-contracts`; `just py-sync-native` rebuilt the editable extension and generated
its actual API stubs. Regeneration and compilation are distinct from behavioral evidence.

## Outcome

**Implemented:** the source modeling kernel now reaches the existing library-owned
mathematics, native algorithms and owned Rust/Python workflows across K4–K7. The whole-root
executable projection, composite tan/log10 handlers, FeOS policy in the generic provider
descriptor, and the old registration-time nested-hint observation path have been removed.
Source fixtures drive the shared conformance harness without model-specific test code.

**Mistakes made and corrected:** a HiGHS feasibility-relaxation result was initially labelled
with a solve termination even though the native API restores the original status, and its
objective retained the original constant. The adapter now reports the operation/status
separately and evaluates the diagnostic copy with a zero objective offset. Nonfinite ranging
and diagnostic values also needed explicit transport classifications. Final targeted controls
cover these cases. The compiler's integrated-lineage test was updated to inspect the shared
rate-stage contract after direct rate evaluation was replaced; native trajectory tests own
its numerical execution. Pure fixture lookup now uses instance identity.

**Deliberate boundaries:** the maintainer retained live FeOS and legacy transient consumers
until their scientific replacements land in K8. Dense SVD, search attempts and diagnostic
samples have explicit budgets. Regime transitions without proof refuse smooth derivatives;
local nonlinear obstruction is not global infeasibility. Dynamic checks cover requested
samples, and terminal integral sensitivities remain an explicit unsupported combination.
Specialized sensitivity/covariance/training workflows and the K8/K9 scientific campaigns
remain outside this execution. Targeted success does not close Plan 21's full acceptance.
