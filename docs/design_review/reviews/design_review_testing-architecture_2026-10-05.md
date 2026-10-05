---
title: Testing architecture
date: 2026-10-05
tier: design
purpose: target
standard: core-3.3
profile: process-simulator-1.4
baseline: 32a14c8abfe50aade0ae5af1021a068ab6c4707d plus inspected uncommitted Plan 25n corrections
evidence: Implemented
decision: Revise
---

# Testing architecture

**Revise the testing architecture for the requested target.** Production already owns
authored physics, relation admission, mathematical preparation, native outcomes,
original-model assessment, candidate permission and durable publication. Testing should
challenge those owners through their contracts, with independent expectations where they
can reveal defects. It should not reconstruct their policy or repeatedly establish the
same fact in every consumer.

The principal problems are package surgery for workload selection, overlapping freshness
checks and execution scopes, unrelated fixture setup, unsound checkout cleanup attribution,
overbroad evidence invalidation, obsolete test shells, and an inconsistent completed
trajectory boundary. The target uses production operations, minimal effect controls,
independent scientific expectations, library-owned runners and one thin evidence owner.

**The premise is a hard design pivot.** Internal APIs, test organization, fixture formats,
report versions and governing policies may change. Historical compatibility, artifact
retention and case counts are not preservation requirements. Delete replaced machinery
with its callers, tests and fixtures. Preserve current physical and behavioral guarantees
because they matter to the product, rather than because an old suite contains them.

## Review contract, coverage and evidence

This is a **Design-tier, target-purpose** review under the selected
[standard](../design_principles/standard.toml): Core/template 3.3, ProcessSimulator 1.4 and
the repository binding. The selector overrides the older profile version mentioned in
the reviewer role file. A fresh design-reviewer formed the independent judgment; the
coordinator inspected decisive evidence and reconciled it with two bounded source maps.

The baseline is HEAD `32a14c8abfe50aade0ae5af1021a068ab6c4707d` plus the inspected dirty
production and test corrections for Plan 25n. This review did not revert, close or resume
that work. Those correction paths were subsequently committed in publication HEAD
`9c953ba490731232bd9eca8b979f8cbee5092989`; the tracked tree is now clean. Observations
remain scoped to the inspected content, not whole-commit qualification. Subsequent changes
must be assessed against the affected observations.

Coverage includes Rust owner-local and integration tests, relation/conformance harnesses,
Python boundaries and workflows, native adapters, scientific references, parity,
performance, generation, governance, tooling and execution/report orchestration. Square
simulation, optimization, dynamics, fitting and studies are considered where these tests
consume production contracts. The survey is broad; it is not an assertion-by-assertion
census or whole-product scientific qualification. Every platform, wheel, solver combination,
workload and tooling module was not examined. Unexamined breadth is not itself a defect.

Diagnoses are **Implemented/source-inspected**, consumed interfaces are
**Interface-checked**, and target corrections and expected benefits are **Proposed**.
No build, test, scientific campaign or timing measurement was run for this review.
Read-only framework version/help inspection and current documentation informed library fit.
There are no new **Tested**, **Measured** or **Formally established** claims.

This principal review owns the combined assessment and stable findings. Until adoption,
the proposed work owner is a future testing-architecture implementation plan. Publication
does not schedule work or automatically add it to Plan 25. Once authorized, that plan
owns finding dispositions and execution evidence; this review retains its observations.

Planning follow-up, 2026-10-05: the maintainer authorized authoring
[Plan 26](../../plans/26-testing-architecture.md), which now owns F01–F09 planning dispositions
and proposed package dependencies. Production execution has not started. This review retains
its original assessment and evidence limits.

## Truth, enforcement and independent evidence

Testing has three distinct responsibilities: verify a production enforcement mechanism,
exercise its use at a meaningful boundary, or challenge its result with an independent
expectation. They are not three independently maintained definitions of validity.

| Guarantee or responsibility | Authority and enforcement | What testing still needs |
|---|---|---|
| Authored physics and parameters | Authored packages; quantity/modeling/compiler admission | Independent physical cases, meaningful malformed declarations and correct binding |
| Layout and local field values | `FieldCheckedBatch`, generated builders/views and raw admission | Constructor/certificate, transformation and hostile raw-input tests at their owners |
| Keys, references, ordinals and domain completeness | Declaration-owned obligations and production candidate/bundle admission | Mechanism-level independent witnesses and actual admission enforcement |
| Mathematical preparation and provider composition | Compiler and `pse-math` operations | Independent values, derivative/layout correspondence and required invalidation cases |
| Native ABI, representation, callbacks and termination | Native adapter consumes admitted problem; library owns numerical algorithm | Small actual native calls, controlled faults and independent expected outcomes |
| Original assessment and scientific permission | Modeling assessment and `workflow::numerics::complete` | Independent validator tests, negative evidence and one meaningful consumer integration |
| Completed result and projection | Completion owner and immutable result representation | Coherent projections, ownership after dropping parents and scoped materialization reuse |
| Workflow and durable effects | Runtime, catalog and operations | Attempt-local cancellation, isolated store/database and fault/recovery boundaries |
| Python translation and transport | Python adapter, generated codec and native boundary | Translation, malformed input, ownership and round trips; scientific duplication only for a distinct risk |
| Emitted-tree freshness | Generator plus filesystem publication/check boundary | One complete freshness check; separate focused generator semantics |
| Selected execution and terminal outcome | nextest/pytest | One inventory and reconciliation boundary with truthful incomplete/failed evidence |
| Qualification and measurement | Thin composition over actual execution and relevant workload inputs | Scope, conditions and provenance; no second implementation of test outcomes |

`FieldCheckedBatch` is not a certificate of every relation invariant. Its private relation,
resolved contract and storage are minted by generated construction or raw admission.
Its documented scope excludes primary keys, foreign keys, ordinal ranges and domain rules.
Generated `View::from_checked` checks the declaration and borrows the admitted owner without
rescanning values; `try_from_batch` admits actual raw schema and values. Retain tests of
these constructors and certificate-preserving transformations. Do not infer bundle validity
from a Rust row type. `pse-relations/force-validate` enables Arrow's `force_validate`, not
scientific permission; correctness recipes retain it explicitly.

Original-model assessment prepares its observation program once per structure and evaluates
the original obligations. Completion composes native facts, checks, closure, applicability,
endpoint and coverage into permission. Consumer tests should use this result instead of
reimplementing general usability. Validator tests must still use independently specified
positive, negative and adversarial expectations; a validator agreeing with itself is not
an oracle.

Completion is the intended permission authority. F08 identifies a trajectory boundary where
writable completed projections can bypass that authority. Seal it before introducing
transport reuse; the positive assessment of other inspected owners does not excuse this gap.

### Physical and numerical preservation constraints

| Inspected model element | Meaning and authority | Independent evidence to preserve |
|---|---|---|
| Dynamic lag and tracking objective | Authored time domain, bounded input and integral in `control-fixtures.pse` | Closed-form exponential trajectory, analytic integral and cross-mode comparison |
| Heat-transfer fitting | Authored measurements, parameters and uncertainty roles | Known fit, rank-deficient case and covariance withholding |
| Flash and phase disappearance | Authored materials/properties and complementarity realizations | Phase-boundary expectations and comparisons of admitted realizations |
| Derivatives and scales | Production derivative demand, providers, coordinates and model scales | Explicit small derivatives or independent differences where warranted |
| Original equations and closure | Original identities and closure obligations | Rejection of failed, missing or incomplete evidence despite native success |
| External reference comparisons | Reference assumptions, conventions and tolerances | Independent behavior at matching conditions; agreement is evidence, not proof |

Variable roles and structural/capability rejection before a solve remain production
responsibilities. Local policy tests supply the actual semantic facts; integration tests
exercise the authored admission chain. A small coefficient or callback oracle deliberately
tests an adapter and does not claim to establish full model admission.

| Numerical stage | Formulation, derivatives and scaling | Outcome owner |
|---|---|---|
| Preparation | Authored guards and realizations; declared derivative demand and physical scales; production class/capability admission | Named readiness/refusal evidence |
| Native execution | Admitted representation and actual controls; library iteration, factors and native state | Typed termination, candidate and source failure |
| Original assessment | Original-coordinate observation program; original bounds, domains and closure tolerances | Structured checks and assessment failures independent of native success |
| Completion and transport | No second solve or formulation; selected numerical and closure policy | One permission decision and coherent derived projections |
| Independent oracle | Explicitly different calculation/reference and comparison tolerance | Can reject a result that production has accepted |

Numerical tests establish their stated cases. This review does not requalify every model's
envelope, closure, structural admission, derivative or outcome mapping.

## Revealing changes

| Scenario | Intended boundary and acceptance |
|---|---|
| S01: Add a model or invariant using existing concepts | One declaration and warranted independent expectations; shared checks follow without copied selection or validation policy |
| S02: Replace a provider/native implementation | Integration owner absorbs ABI/representation/capability changes; consumers retain semantic outcomes |
| S03: Test admission or permission locally | Actual semantic inputs and explicit controls, without unrelated package, solver or store setup |
| S04: Compose a study/dynamic/fitting journey | Intact definitions and shared preparation/execution; failures and attempts stay separate |
| S05: Change a generated boundary representation | One freshness owner plus distinct codec, admission and transformation tests |
| S06: Reuse evidence or measure after unrelated prose edits | Context remains recorded; only actual claim inputs determine reuse eligibility |
| S07: Run while another agent edits | Tests own temporary effects; checkout observations do not invent causation |
| S08: Read several trajectory relations | One coherent completed result, bounded transport work, retained leases and no test-side cache |

A genuinely new physical concept may require several production owners to change. That
does not justify repeating existing policy for another instance of a known concept.

## Findings and corrective design

### <a id="f01"></a>F01 — Workload selection rewrites and reconstructs the authoritative package

**Principles:** AP-01/AP-03/AP-06, DP-06/DP-08/DP-16, PS-11/PS-13. **Scenarios:** S01/S04.

**Implemented/source-inspected:** `tests/support/plan14.rs::reference_documents` parses
authored files, retains a hard-coded test-ID set, walks descendants to delete remaining
tests and renders modified documents. Dynamic optimization reparses control fixtures and
restores selected declarations; complementarity acceptance similarly restores removed tests.
Production already supplies `ModelingFixtureSelection::Selected` and `declared_execution`.
An ordinary fixture addition therefore requires knowledge of package membership and test-owned
stripping/restoration, and tests load a different package revision from normal consumers.

**Proposed:** load intact documents through production loading and select execution through
production declaration identities. Shared support may locate inputs and supply explicit
resource limits. It must not own a descendant walker or selector disguised as authoring.
Add a narrow missing production selection operation only if necessary. Deliberately malformed
documents remain valid test inputs; they are not workload-selection surgery.

**Verification:** an ordinary new fixture needs no whitelist/restoration change; selected
coverage stays distinct from package coverage; unknown selection is refused. Delete filtering,
restoration and their replaced artifacts.

### <a id="f02"></a>F02 — Generated freshness is established repeatedly

**Principles:** AP-01/AP-03, DP-10/DP-16/DP-23. **Scenario:** S05.

**Implemented/source-inspected:** governance `codegen_regeneration.rs` generates
`Language::ALL` and compares expected bytes with the checkout. `xtask::codegen::run` calls
the same generator but also checks output roots, relevant generated additions, Python candidate
validity and missing/extra/untracked files. Workspace tests, separate governance selection
and generation recipes overlap. Existing recipe-leaf deduplication does not merge independent
implementations of the same freshness assertion.

**Proposed:** keep one authoritative emitted-tree comparison for the required outputs and
remove the weaker overlapping full-tree test. Keep focused generator semantics, meaningful
determinism checks, query preparation, document schemas, native bindings and compiled Python
surface checks where they establish different contracts. Do not replace the strongest check
with the weaker expected-file loop.

**Verification:** one check owns each artifact property. Missing, extra, stale and invalid
candidate output still fails. Retained generator tests detect distinct transformation defects.
Standard invariant witness artifacts are addressed separately in F09.

### <a id="f03"></a>F03 — Broad execution configurations repeat obligations and report processing

**Principles:** AP-01/AP-03/AP-05/AP-06, DP-10/DP-16/DP-23. **Scenarios:** S02/S03/S05.

**Implemented/source-inspected:** `validation_scope.comprehensive` selects full default and
native Rust workspaces; governance is selected separately as well. Python unit/component
scope overlaps linked Python scope. Default/native feature differences are real, but the
composition does not establish why every common deterministic test needs both executions.
The assessment and native runner also repeat explicit inventory/provenance enumeration;
native Python and its enclosing assessment parse the same terminal report.

**Proposed:** common obligations execute once in the selected production configuration;
feature-absence/different-capability checks run only in modes whose behavior they establish.
Native adapter/workflow tests and Python boundary tests retain their necessary modes. Full
configuration matrices remain explicit experiments rather than default repeated obligations.
Use existing filters, profiles, groups and markers, not a new test registry or impact engine.
One owner inventories and reconciles each invocation; native execution supplies actual binary
and link provenance to that owner.

**Verification:** one explicit inventory and terminal reconciliation per invocation; no
unexplained repeated common obligation. Missing, duplicate, unexpected, failed and unexecuted
selected cases remain visible. Identity alone does not prove equivalence across feature graphs.

### <a id="f04"></a>F04 — Global fixture and package policies impose unrelated setup

**Principles:** AP-01/AP-06, DP-10/DP-17/DP-19, PS-13. **Scenarios:** S01/S03.

**Implemented/source-inspected:** Python's session-autouse `registered_extension_types`
performs all-extension IPC even for ndarray refusal and stub tests. Session scope is local
to a worker process. Nextest groups whole conformance/lifecycle packages under resource
premises also applied to small query/canonicalization checks. Python cost markers declare
units solver-free, while some current unit-labelled workflows invoke native execution.
For example, `test_study_repeated_bindings_retain_distinct_occurrences_and_owned_results`
constructs a runtime and executes an explicitly selected Ipopt study; other unit-marked
study cases execute POUNCE. Historical markers cannot define the execution contract when
they contradict consumed dependencies.

**Proposed:** make all-extension registration one explicit boundary check or required fixture
dependency. Classify execution by actual effects and resources. Keep native/database caps for
tests that need them, and keep small policy/codec tests local. Preserve `pse-testkit`'s bounded
engine support and owner-local `SharedRuntime::build` composition at their different dependency
levels. Do not turn the lower-level testkit into an always-initialized workflow fixture.

Reuse immutable data/preparation within an appropriate journey. Mutable runtime, cancellation,
database and attempt state remain separately owned. Nextest's process-per-test model means a
process-local singleton cannot share preparation between tests. Prefer one aggregate journey
with distinct assertions when its shared setup is justified, rather than a global cache.

**Verification:** focused codec/stub tests do not run unrelated IPC or publication. Small
checks do not inherit full scientific resource assumptions. Fault/cancellation cannot pollute
other attempts, and cost selection reflects actual operations.

### <a id="f05"></a>F05 — Git polling cannot enforce or attribute cleanup

**Principles:** AP-02/AP-05/AP-06, DP-18/DP-19/DP-23. **Scenario:** S07.

**Implemented/static counterexamples:** `VerifyCleanup.pytest_runtest_protocol` compares
untracked paths and tracked porcelain-status lines around each test. An agent editing a clean
file during a test is attributed to that test. Editing an already-dirty file can leave its
status line unchanged and escape detection. Four global Git queries per test also repeat
inspection; this is an operation count, not a timing measurement.

**Proposed:** delete per-test global attribution. Use owned temporary directories, explicit
output roots and isolated stores/databases. Test a tool's prohibited-write contract against
an isolated fixture, with an enforceable boundary where required. Any operator checkout
observation must disclaim causal attribution. Cleanup must never reset another agent's work.

**Verification:** concurrent authorized edits cannot fail unrelated tests. Dedicated effect
tests detect prohibited writes to existing/dirty inputs. Ordinary tests no longer poll Git.

### <a id="f06"></a>F06 — Global source sealing overcouples evidence and measurements

**Principles:** AP-01/AP-02/AP-06, DP-09/DP-10/DP-22/DP-23. **Scenario:** S06.

**Implemented/source-inspected:** `validation.py::SOURCE_PATHS` covers product code, tests,
tooling, plans, reviews and architecture prose. Unchanged-input reuse requires equality of
this whole set. `case_measure.require_functional` requires the full comprehensive scope and
the same global source set. An unrelated review edit consequently requires broad reassessment
or manual transfer before a selected numerical measurement can proceed.

**Proposed:** separate contextual snapshot from inputs determining a claim. Use minimum
trustworthy explicit scopes at existing recipe/workload boundaries, including command,
features/profile, relevant code/data/policy, locks and native identity. Require the actual
functional prerequisites of a measurement, rather than every comprehensive check. Preserve
reviewed transfer with original provenance when uncertainty requires it; do not automatically
approve changed-code reuse or invent a dependency-impact framework. Executable policy or
workload documentation can be relevant; unrelated prose is not automatically a numerical input.

**Verification:** irrelevant prose changes do not invalidate a stable claim. Relevant model,
policy, lockfile, workload or binary changes do. Reuse and transfer remain distinct from fresh
execution, and unresolved dependency scope remains conservative.

### <a id="f07"></a>F07 — Bootstrap shells and upstream-only probes remain product obligations

**Principles:** AP-01/AP-06, DP-16/DP-23, PS-13. **Scenarios:** S01/S02.

**Implemented/source-inspected:** three `phase0_placeholder` tests only assert their crate
names. The structural test crate contains only its placeholder; real structural behavior is
tested at the production owner. `math_composition.rs` largely calls Symbolica, faer, num-dual
and FeOS directly, bypassing production composition/provider adapters. Such probes can pass
while production lowering is wrong. Existing production derivative and assembly tests provide
the appropriate integration boundary. `tests/support/physical_source.rs` has no discovered
repository consumer in the source search.

**Proposed:** delete placeholders and proven-unused support; retire empty crates through the
decision route. Remove upstream-only probes from product qualification. Retain a probe in
capability research only if a current question warrants it, without automatically copying
every old test. Add production-adapter regression coverage only for an identified gap. Actual
ABI/link assumptions and native fault controls remain useful.

**Verification:** remaining tests identify a current production, scientific, boundary or
tooling failure. No historical suite name or test count is preserved as a requirement.

### <a id="f08"></a>F08 — Trajectory completion is mutable and transport repeats whole-result work

**Principles:** AP-01/AP-02/AP-04/AP-05/AP-06, DP-01/DP-05/DP-09/DP-10/DP-19, PS-10/PS-12.
**Scenarios:** S04/S08.

**Implemented/source-inspected:** `ModelingTrajectory` exposes mutable public `accepted`,
`checks`, `checks_complete`, `reports`, `report`, `prepared` and identity fields alongside
private completion. `diagnostic()` returns no failure when the public `accepted` flag is
true; transport also carries the private completion assessment. Legal Rust mutation can make
these interpretations disagree. The current constructor deriving acceptance from completion
does not enforce that relationship for the lifetime of the returned object.

Python `ModelingTrajectory.table` delegates to `NativeModelingTrajectory::table`, which calls
`tables()` and removes one requested relation. `tables_for_kind` constructs and finishes the
whole collection of samples, sensitivities, events, checks and reports each time. The event/
terminal-report boundary test requests five tables, causing five whole materializations by
the inspected call graph. This count is source-derived; no duration was measured.

**Proposed:** seal completed identity, report, checks and permission behind construction and
read-only accessors. Derive acceptance and diagnostics from the single retained completion.
Then give this production result a bounded materialization owner, analogous to `RunResult`,
or a requested-relation-only encoder if retention cost makes that better. Any reuse must bind
the actual computation kind and immutable source, preserve allocation leases and account for
retained memory. New attempts remain distinct. Do not solve this with pytest caching.
The export kind must distinguish Simulation from Shooting. A transient failed export must
not become a permanently cached refusal unless that is the explicit result contract;
resource-refusal and retry behavior belong to the transport owner.

`RunResult::tables` already uses its result-owned `OnceLock`, and `table` clones checked
batches. It needs no parallel cache or equivalent correction.

**Verification:** callers cannot independently overwrite completed permission/report meaning.
Accessors and exported assessment agree. Multiple relation reads do not repeatedly rebuild
unrequested tables, remain safe after parent handles drop, and cannot reuse across a different
computation kind or attempt. Measure speed only if claiming a quantitative benefit.

### <a id="f09"></a>F09 — Per-declaration witness artifacts repeat generated invariant families

**Principles:** AP-01/AP-03/AP-06, DP-06/DP-10/DP-16/DP-23. **Scenarios:** S01/S03/S05.

**Implemented/source-inspected:** `pse-schema::builder::integrity` derives primary/unique
key, foreign-key and nested occurrence queries from the declaration-owned obligation product.
`fixture::generate::pair` constructs standard witnesses by those families, while the dynamic
conformance harness requires valid/violating YAML and separate executions per invariant.
`pse-rules` already binds every declared query against its exact catalog inputs and tests typed
duplicate findings. Ordinary relation additions acquire more stored witness artifacts and
executions even when they introduce no new invariant algorithm.

**Proposed:** verify generated mechanisms by distinct semantic shapes and independently
specified witnesses, plus catalog-wide binding/declaration checks and focused real admission
tests. Cover singleton/composite keys, nullable uniqueness, self/composite/nested references,
ordinal edges, field representations and exact diagnostic identities where behavior differs.
Retain independent witnesses for genuinely authored/custom semantic queries. Derive standard
test inputs at the appropriate test owner; remove mechanically regenerated YAML and its
freshness/inventory obligations. Merely regenerating every pair at discovery removes files but
does not by itself streamline repeated execution.

This does not make relational validity intrinsic to field construction, remove runtime
enforcement or compute expected answers by running the same SQL. Different semantics require
different tests; a common generated family does not require one duplicated proof per instance.

**Verification:** a new instance of an established generated family adds no bespoke witness
code/files or unnecessary repeated case. A new semantic query still has a discriminating
oracle. Adversarial family cases and real candidate admission catch deliberately wrong query
projection, null/reference handling or missing enforcement. Replace only cases whose distinct
risk is established as covered; catalog binding alone is not behavioral verification.

Family identification must follow the production producer and obligation contracts, without
a test-owned whitelist, second invariant registry or bespoke query analyzer. Establish the
consequential variation coverage before deleting witnesses. Deliberately untrusted inputs
must still reach the query under test when earlier admission would prevent exercising it.

## Retention and deletion decisions

| Current example | Direction and reason |
|---|---|
| Placeholder tests and empty structural shell | Delete; no current behavioral claim |
| Test-owned package stripping/restoration | Delete; intact production selection replaces it |
| Weak full-tree freshness comparison | Delete after retaining the stronger emitted-tree boundary |
| Generated invariant YAML/inventory mechanics | Delete for standard families; retain genuinely independent authored data |
| Upstream-only math capability probes | Remove from product qualification; research only where a current question exists |
| Repeated canonicalization assertions | Consolidate by risk; preserve framing, masks, slices, dictionaries, signed zero, visible changes and reservation obligations |
| Certificate/raw-admission/checked-transform tests | Retain centrally; local evidence is not bundle evidence |
| Custom invariant-query witnesses and admission integration | Retain distinct purposes and independent expected keys |
| Native callback fault oracles | Retain minimal trial/fatal/panic/cancellation controls at the adapter |
| Dynamic closed forms and cross-mode comparisons | Retain independent physics/realization evidence |
| Unidentifiable fit and covariance withholding | Retain distinction between prediction and qualified derived estimates |
| Python IPC, ownership and malformed nested values | Retain real translation risks; remove unrelated autouse setup |
| External scientific comparisons | Retain where they establish a current claim under matching conventions/tolerances |
| Historical enumeration/name compatibility | No automatic retention; keep only a deliberate current product contract, otherwise retire through its authority route |
| Performance smoke and campaigns | Preserve readiness versus measured performance; move scale-only work out of ordinary functional checks |
| Tooling negative controls | Retain meaningful selection, failure, stale/partial output and protection boundaries |
| Existing result-owned table reuse | Preserve; no second testing cache |
| Trajectory whole-map materialization per relation request | Replace at the production result owner, with bounded demand-aware transport |
| Mutable completed trajectory permission/evidence fields | Seal behind read-only access, preserving one completion authority |

A test of an intrinsic guarantee can be removed once its claimed failure is impossible
under the actual consumed construction contract. Retain focused verification of that
construction/enforcement owner and boundary bypass paths. Do not blanket-delete tests
because a validator exists, a schema is generated, or an upstream library is reputable.

For expensive journeys, use enough distinct points to test binding, reuse, refusal and
failure isolation. Keep scale/scientific campaigns separately when scale itself is the
claim. For example, a thousand-point sweep is not required merely to prove that a value
change reuses preparation; its scale experiment can remain explicitly selectable.

## Library fit and alternatives

The inspected local tools are nextest 0.9.146, pytest 9.1.1 and pytest-xdist 3.8.0.
Existing nextest selection, groups, JSON inventory and JUnit support, and pytest fixture
lifetimes, are suitable mechanisms. Current documentation was consulted through Context7
and official pages; it does not qualify every feature at the locally resolved version.
No extra dependency pin or upgrade is a prerequisite for this review.

Nextest owns selection and group scheduling ([selection](https://nexte.st/docs/selecting/),
[groups](https://nexte.st/docs/configuration/test-groups/)), process isolation
([execution model](https://nexte.st/docs/design/how-it-works/)), and inventory/report formats
([listing](https://nexte.st/docs/machine-readable/list/),
[JUnit](https://nexte.st/docs/machine-readable/junit/)). Pytest owns fixture creation,
cache and teardown within its scope ([fixtures](https://docs.pytest.org/en/stable/how-to/fixtures.html)).
Thin domain reconciliation can still be necessary for selected-versus-terminal completeness
and native/source provenance; these are not all replaced by a runner exit code.

Proptest, Criterion and the existing production data/numerical libraries provide established
mechanisms already used here. The review does not require replacing them or implementing
another property generator, benchmark sampler, scientific validator or test runner. Online
record/replay features are not treated as locally qualified replacements for the evidence
contract; replay also cannot become fresh execution by renaming it.

| Alternative | Judgment |
|---|---|
| Keep current machinery and add a universal test registry | Reject: adds another representation while retaining the underlying duplication |
| Run every test through a full runtime/package | Reject: obstructs local policy, codec, raw-input and callback tests |
| Derive every expected result from production assessment | Reject: loses independent detection of wrong physics/transformation |
| Keep independent ad hoc fixtures in every suite | Reject as the default: repeated semantic and lifecycle decisions persist |
| Production operations with focused independent oracles and minimal effect support | Select: correct responsibility boundaries and ordinary extension locality |
| Existing framework execution with one thin evidence composition | Select: the simplest viable execution alternative for the same target |

Expected latency and maintenance improvements are **Proposed**. Removing repeated operations
reduces those operations by construction. End-to-end speed and memory effects, including
the cost of retained trajectory tables, remain unmeasured.

## Foundation and gate assessments

| Foundation | Verdict and decisive evidence |
|---|---|
| AP-01 Separation of concerns | **Violated:** selection, freshness, setup, execution and evidence boundaries in F01–F04/F06–F09 |
| AP-02 Stable contracts | **Violated:** cleanup attribution does not establish its guarantee; measurement prerequisites overcouple unrelated inputs; mutable completed projections break coherence (F05/F06/F08) |
| AP-03 Composition | **Violated:** package reconstruction and repeated checking/execution/materialization (F01–F03/F08/F09) |
| AP-04 Domain model and semantic authority | **Violated at the completed trajectory boundary:** mutable acceptance/report projections can disagree with private completion (F08). Inspected authored physics, relation evidence and original assessment remain useful authorities |
| AP-05 Explicit structure and constraints | **Violated:** configuration-based repetition, unenforceable global effect attribution and mutable completion projections (F03/F05/F08) |
| AP-06 Local reasoning/testability | **Violated:** unnecessary package reconstruction, setup, ambient checkout dependence, broad qualification and repeated transport work |

G9 follows these individual judgments; strengths do not offset violations.

| Gate | Judgment, evidence and scope limit |
|---|---|
| G1 Authority | **Fail:** F08 allows a completed permission projection to diverge without reconciliation. Independent oracles themselves are not competing production authorities |
| G2 Semantic fidelity | **Fail at F08:** accepted/diagnostic interpretation can disagree with completion. Local field versus bundle evidence and selected versus package coverage remain explicit distinctions |
| G3 Validity | **Fail for F05:** checkout status can miss prohibited modification and reject unrelated work; no blanket physical admission failure is established |
| G4 Hidden behavior | **Pass, Interface-checked in inspected support:** construction and fault/observation effects are identifiable; unnecessary setup remains an architectural defect |
| G5 Consistency/recovery | **Pass, Interface-checked for inspected attempt/evidence architecture:** retained incomplete/failed states and attempt ownership are preservation constraints; recovery campaigns were not run |
| G6 Transformation/reuse | **Interface-checked for existing checked transformations and RunResult reuse; unresolved for proposed trajectory reuse:** immutable inputs, export-kind scope and lease-preserving transport are prerequisites; a cache alone is unsound |
| G7 Truthful capability claims | **Unresolved beyond inspected contracts:** projections in F08 need correction; placeholders/probes do not qualify production. No current full-campaign or performance claim is made |
| G8 Library leverage | **Pass, Interface-checked with simplification required:** adopted runners and production libraries own generic mechanisms; no bespoke test numerical engine was established |
| G9 Architectural fitness | **Fail:** representative changes expose the foundation violations above |
| PS-G1 Physical consistency | **Unresolved as product qualification:** checks/oracles exist; no complete physical campaign was run and no general physical violation is asserted |
| PS-G2 Well-posedness | **Unresolved as product qualification; Interface-checked mechanisms:** structural/capability admission remains production-owned; not every mode/model was executed |
| PS-G3 Numerical integrity | **Unresolved as product qualification:** original assessment is independent of native status. F08 establishes result-API integrity counterexamples, not a wrong numerical trajectory from an unmodified nominal run or a demonstrated invalid durable publication path |

PS-01–PS-12 constrain preservation of current physical, formulation, derivative,
initialization, solver and result meaning. PS-13 supports shared production checks with
independent scientific evidence. It does not require another testing implementation of
those checks. No SHOULD exception is requested.

## Authority routes and dependency order

Blueprint §24.1/§24 and the qualification guide must describe the adopted responsibility
boundaries, scoped evidence and execution composition. Cleanup requirements should express
owned effect boundaries rather than impossible per-test Git attribution. Historical layer
names, report schemas and enumeration compatibility are not target constraints. Update
architecture through the decision/design route with its revision row; supersede accepted
ADRs where necessary rather than editing them. Crate removal and governance/qualification
policy changes use their applicable ADR and review routes. Ordinary test retirement and
refactoring within accepted contracts need no invented compatibility stage.

The proposed testing-architecture plan should own F01–F09 only after adoption. Its work
order follows capability dependencies:

1. Settle truth scopes: local field versus relational admission, native facts versus original
   assessment/permission, independent expectation versus production check, and runner outcome
   versus qualification evidence. Define relevant measurement prerequisites.
2. Seal completed trajectory meaning and choose bounded production transport reuse. Supply
   only necessary narrow production selection/construction/observation seams.
3. Replace test-owned package composition and effect attribution. Remove strip/restore and
   Git polling; preserve deliberately untrusted/fault inputs.
4. Consolidate emitted freshness and invariant families around their unique risks. Delete
   superseded checks, standard generated witness artifacts, empty shells and unneeded probes.
5. Compose responsibility-specific execution, fixture scopes, resource groups and evidence
   inputs. Keep one inventory/reconciliation owner and explicit mode-dependent obligations.
6. Qualify the landed design after functional scope, using affected targeted checks during
   implementation and relevant integrated/native/Python/reference/performance and static/manual
   checks once at scope end. Old case counts are not acceptance criteria.

Consequence priority differs from order: trajectory authority and cleanup attribution have
direct integrity consequences. Truth/input contracts enable safe execution consolidation;
the completed-result boundary is a prerequisite for sound trajectory reuse. Proven-empty
shells can retire independently after their decision route is satisfied.

## Verification and decision

| Claim | Evidence and closure criterion |
|---|---|
| Selection and freshness duplication exists | **Implemented/source-inspected:** delete reconstruction/weak checks while retaining intact selection and strongest publication checks |
| Cleanup attribution is unsound | **Implemented/static counterexamples:** isolated effect tests detect actual prohibited writes and permit unrelated concurrent edits |
| Completed trajectory truth and transport need a production correction | **Implemented/source-inspected:** read-only completion projections agree; table access has one bounded materialization owner |
| Generic invariant families need different evidence from authored semantic queries | **Implemented/source-inspected; correction Proposed:** independent adversarial shapes plus catalog binding and real admission replace only equivalent instance repetition |
| Scientific/certificate tests remain necessary | **Interface-checked:** retained tests challenge constructor, validation, translation and physics with distinct expectations |
| Target reduces repeated work and extension burden | **Proposed:** demonstrate deletion and ordinary additions inheriting checks without repeated semantic decisions |
| Target is faster or scientifically qualified | **Not established:** named representative execution/measurement under stated conditions is required for those claims |

**Behavioral/semantic adequacy:** cleanup and completed trajectory contracts require revision;
whole-product physical/numerical qualification remains outside the evidence established here.

**Architectural fitness:** fails G9 through concrete responsibility and authority defects.
Existing production assessment, bounded fixture construction, independent oracles and retained
`RunResult` transport are preservation constraints, not reasons to retain surrounding machinery.

**Overall decision: Revise.** The correction is a production-owned, minimally supported testing
architecture with independent expectations and truthful scoped execution evidence. It is
**Proposed**, not implemented or qualified by publication. Subsequent review-derived plans
must execute the hard pivot and delete replaced mechanisms without compatibility layers or
historical-artifact retention.
