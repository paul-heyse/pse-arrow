---
title: "Plan48h: Qualification and design publication"
status: done
date: 2026-10-20
adrs: []
review_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md, docs/design_review/evidence/execution-performance-recommendations-audit-2026-10-20/recommendation-register.md]
scenario_sources: [docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md]
doc_role: plan
doc_owner: docs/plans/48-execution-performance-and-numerical-efficiency.md
doc_topics: [validation, numerical-execution, performance, documentation]
---

# Plan48h: Qualification and design publication

## Purpose, baseline and boundaries

Establish that the selected Plan48 changes compose, remove their intended unnecessary work
and preserve scientific/ownership meaning. Publish the implemented design and reconcile
other plan documents **after** this series has been implemented and qualified. The
[coordinator](48-execution-performance-and-numerical-efficiency.md) owns all findings,
accepted RC01–RC12 boundaries, I01–I36 outcomes and completion status. This companion owns
only H01–H04 execution. The independent
[Completion audit](48-execution-performance-and-numerical-efficiency.md#completion-audit)
identified CA01 and CA02, whose separately authorized repairs now pass targeted current-source
acceptance and all three full Clippy modes. H01/H03/H04 are complete for the adopted scope.
H02's bounded observations and I36 decision remain applicable. Prior selected native/Python
acceptance and publication retain their original conditions.
No product tests or new simulation measurements were run during plan authoring.

The current foundation is adequate: `.config/process-cases.json` and
`.config/preparation-cases.json` declare existing workloads; `scripts/pipeline_review.py`
selects explicit cases, consumes prebuilt binaries and runs cooperative stop/cleanup via
the existing case-audit owner. Existing test, native, Python, diagnostic comparison and
offline pack operations provide acceptance surfaces. Extend a declaration/control only
where the new behavior needs one; do not build another campaign runner or proof system.
Blueprint §24.2 and the [qualification guide](../dev/validation-assessment.md) own command
meaning and exclusions.

The audit already corrected the interpretation of the original campaign. R2 unset
diagnostics did not mean off; LLDB thread stacks were not exact on-CPU shares; outer
minus invocation was not exclusive preparation; cold/seeded native records were separable.
The new comparisons must respect those corrections instead of reusing their invalid
labels. Historical records remain valid evidence of their own conditions, not evidence
that new code ran. There is no required all-case rerun, exact environment match or claim
that one small sample establishes industrial capacity.

## Acceptance design

### H01 — source-to-package completeness and revealing controls

Check the real migrated producer/consumer closure for AF01–AF18/N01–N04 and all 162
register rows. Consume the coordinator/coverage appendix, current source and actual package
acceptance. Verify that shared findings close only after all their obligations, such as
AF12's separate KINSOL and cone controls, are complete. Omitted proposals are not missing
implementation. Existing capabilities identified by the audit are not new deliveries.

For each actual replacement, inspect consumers and delete its legacy implementation,
callers, fixtures and obsolete tests as soon as targeted checks establish replacement.
Do not preserve a second production route as evidence. A selected native alternative may
coexist as a real capability with its own explicit eligibility; that is distinct from
keeping superseded plumbing. Reconcile package dependencies with delivered behavior,
including receiving binding, complete linearization, neutral reset and terminal completion.

Targeted acceptance belongs to the package that introduces behavior. H01 identifies any
missing test scenario and returns it to that owner; it does not defer all consumer migration
or regression testing until the end. Use independent expected lexical/physical outcomes
where a production resolver or transformation could reproduce its own error. A copied
counter or successful serialization is not independent scientific assessment.

### H02 — short complete-operation observations and follow-up decision

Use explicit selected cases, existing controls and source/work counters to answer a named
question. Start with one ordinary observation per selected case after required untimed
setup. Capture detail/profile only when ordinary observations leave a concrete question.
Every condition explicitly sets its effective diagnostic mode **before process/import
initialization**; `PSE_DIAGNOSTICS=off` is off, while unset is ordinary-enabled. Selected
recording/capture/export are separate settings. Keep thread widths, start policy and actual
native realization in the descriptive context; do not demand environment identity proof.

The initial selection is by changed capability, not the Cartesian product of all libraries,
methods, dimensions, observer modes and widths:

| Changed capability / question | First revealing existing cases or targeted controls | Observations and preservation |
| --- | --- | --- |
| Source checking, registry and validation | Small synthetic scope/import/edit controls; `pr-inline`, then `pr-nested` or `pcsaft-02` if the changed path needs them | Examined names/dependencies, actual domain/description preparation counts, cold admission and warm edits separately; same identities/refusals |
| Contributor/sparse action work | `compact-root-banded-4`, `compact-root-banded-8`, `compact-implicit-derivatives-4-2`; sparse HVP/provider controls | Entries/directions/reconstructions/factors and retained scratch; current provider/branch/demand and original results |
| Compiled module/artifact selection | One existing co-used scalar body with actual compilation selected; targeted named-export/control fixture | Compiler children/setup versus warm evaluation, demand/orders/layout and private scratch; not capability-flag inference |
| Root/cone control corrections | Shifted/scaled native controls; `review-root-banded-64` only as a bounded specific regression; `review-conic-soc`, `review-conic-psd` and one POUNCE-convex control | Native status/norm/work versus original acceptance; Limit remains distinct from physically invalid; no universal one-retry promise |
| Starts and native retained state | Two-variable Ipopt exact/limited-memory, POUNCE exact/limited-memory and Uno accepted-start cases, plus a small parameter perturbation | Cold/seeded attempts separately; primal/optional payload/reuse receipt truth, iterations/evaluations/factors and actual reset/discard |
| Recycles and grouping | `compact-recycle-basic`, `compact-recycle-coupled`, `compact-recycle-stagnating`; tiny grouped/ungrouped control | Actual map/domain/branch/root eligibility, history/work and original closure; no universal n≈32 or Anderson history |
| Dynamic allocation/products | `compact-idas-ode-5`, `compact-idas-dae-5`, smooth/stiff controls as relevant; selected Jacobi/coupled-preconditioner and failure-reset fixtures | Native creation/reinit/symbolic/numeric counts, selected derivative entries, trial validity, local error and original trajectories |
| Fitting and anchored windows | `review-sparse-fit-2-1`, then size8 only if needed; short two-window fixed-trial fixture and selected objective-only/gradient controls | Integrations/derivative demand, GN constraint curvature, selected axes/checkpoints, private lanes and ordered continuity products |
| Study dispatch/capture/durable work | Four-point 1/4-worker compact/full/continuation cases; existing managed-durable short case only for affected store integration | Compatible dispatch, ready visits, sibling progress under held producer, original assessment, continuation reuse, claims/ACK/drain |
| Diagnostics and aggregate publication | History-zero/current/final native controls, loss/overflow/phase-exit controls; retained JSON/IPC scalar inputs for streamed publication | Effective mode/cadence, exact local work, current-state truth, source pointers/nulls/counts and coherent final files |

Do not run every listed case when a smaller control settles the changed mechanism. Do not
substitute a wall-time pass for unchanged science. Source counts and constructed products
can establish work removal without a statistically significant timing result. Separately
observe cold first-result, warm re-solve, accepted throughput and selected result availability;
do not sum overlapping native intervals, compare incomparable populations or call all
outside-invocation time preparation.

For a qualified method that removes work on first principles, prefer adoption and diagnose
unexpected composed costs rather than requiring a small timing win first. Marginal or noisy
timing differences do not reject it. A slowdown above roughly2× warrants causal investigation;
it is not a universal rejection gate and does not justify changing accuracy or ownership.
When a default/algorithm choice genuinely depends on latency, use five alternating matched
observations per candidate after untimed setup and report actual medians/ranges and work.
If that short series is inconsistent or a case exhausts its budget, stop the selection,
state the uncertainty and use I36; do not repeatedly enlarge the campaign until a winner
appears. Five observations are descriptive, not a universal significance or tail-latency proof.
No exact speed, exponent, callback-time, RSS ratio, iteration-ratio or core-scaling gate is added.

Keep exploratory science near the existing **60-second cooperative deadline**, followed by
actual registered cancellation/drain. Never terminate unrelated owners or classify interrupted
science as convergence. Preparation, execution, publication and cleanup remain distinct.
The existing prebuilt-binary driver does not build an extension behind a captured run.
Use ordinary fast development builds for exploration and explicit validated correctness
extensions for assertions. Full wheel installation is not required.

**I36 finite decision:** after the short selections, identify any remaining mechanism that
small cases cannot expose: sparse fill/coupling, retained working-set pressure, large-batch
transfer economics or long-horizon integration behavior. State the smallest composed case,
distinctive parameter/size, diagnostics, expected discriminating outcome, budget and actual
cleanup route. A 2,000–5,000-equation flowsheet is a possible later experiment, not a required
Plan48 benchmark or an already authorized implementation. The inquiry closes with either
no currently justified longer work and a concrete reopening trigger, or a scoped follow-up
proposal for later review/planning. Broad follow-up execution is not silently added here.

### H03 — assembled behavior and final quality

After functional packages and selected conditional adoptions are complete, execute the
relevant integration, component, native/Python and managed journeys once, broadening only
for a new change/failure/unresolved concern. The assembled paths must cover source edit →
prepared reuse → native solve → independent original assessment → demanded result, plus
affected study/recycle/dynamic/fitting and offline-review paths. Include one revealing
failure/cancellation/partial-publication journey for changed ownership. Use initialized
authenticated fixtures only for tests that actually need a store; pure scopes retain
their actual dependencies. A missing required solver/capability is a failure, not a skip.

Select recipe/filter names from the current owners at execution; do not promise that a
historical test identity still exists. The intended command routes are:

- During packages: `just check-package`, `just unit-package` and affected explicit native
  correctness routes; generators run `just codegen` when declarations change.
- At functional scope end: `just hygiene`, which includes Python Ruff/Pyrefly, generated
  drift and default/no-default continuing Clippy; fix failures and rerun the failing recipe.
- Also select the linked native-solvers Clippy closure with `--keep-going`, workspace/all
  targets, `-D warnings` and explicit validation through `scripts.arrow_validation`; a
  default/no-default pass does not cover opt-in native code. Keep all three modes distinct.
- Run manual `just governance`, `just docs`, applicable `just lint-native-contracts` /
  `just lint-native-data`, and `just features-powerset` / affected feature checks. Run
  affected scientific parity against the pinned reference where the changed scope consumes it.
- Rebuild only the needed editable Python extension through the fast validated route for
  correctness. No full wheel, alternate toolchain, release campaign or provenance proof
  follows merely from changed environment bytes.

Report actual commands/modes/scopes/results against the **zero** failure/warning baseline.
Targeted, static, native assembled, measured and unexamined scopes stay distinct. Original
failed/interrupted outcomes remain visible when a repair passes; tests that never ran are
not passed. No command is introduced as a commit/push/merge prerequisite.

### H04 — enduring publication and reconciliation

After accepted behavior is implemented and qualified, move enduring meaning to its owning
architecture/development guides through the appropriate ADR/design route and blueprint
revision row. Record selected native defaults, unsupported alternatives, compatibility/reset
and observation meanings where their consumers look. Accepted ADR history stays immutable.
Dependency pins move only if a selected capability actually requires it; no wholesale resolve.

Then review existing active/retained plans for changed dependencies, replaced scope and
qualification references. Update their native owners rather than copying live status into
Plan48. This future reconciliation does not restart or close unrelated Plan28/32/40 scope.
Retire completed plans/reviews only after enduring meaning and actual inbound consumers
are accounted for, using lifecycle preflight; Git history remains the archive. Do not add
an archive tree or delete raw campaign evidence merely because Plan48 is complete.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| H01 | Actual deliveries and selected inquiry outcomes from A–G | Reconcile every finding/register row, inspect real migrated consumers and replacement deletion, return missing targeted controls to their owner | done: repaired selected boundary and public BBD consumers independently reviewed; CA01/CA02 settling and adjacent controls pass |
| H02 | Correct native receipts/controls and reliable current-phase semantics for changed paths; relevant packages ready | Run the smallest revealing short selections, document descriptive work/latency and settle I36's next-stage question without an automatic long campaign | done: package Outcomes retain actual bounded work/latency and scientific limits; I36 finite follow-up decision is settled below |
| H03 | Functional scope and selected conditional adoptions complete; H01/H02 interpretation available | Pass selected assembled/scientific/ownership journeys and scope-end checks, including all three continuing Clippy modes, with exact zero-baseline reporting | done: current 27-test math/compiler/native/runtime selection and all three continuing workspace/all-target Clippy modes pass; original selected acceptance retained |
| H04 | H03 selected acceptance and settled decision outcomes | Publish enduring contracts through their decision owners, then reconcile existing plan documents and lifecycle references without changing unrelated scope | done: repair outcomes, completion audit and current-work/lifecycle records reconciled; existing enduring target unchanged |

H01 is a continuing completeness responsibility, not a reason to postpone ordinary tests.
H02 can run a selected completed path before all unrelated packages finish. H03/H04 are
the final assembled/closeout stage; document ordering does not require repeating campaigns.

## Outcome (recorded after implementation)

### Short-round follow-up decision

**Measured and source-inspected, bounded:** the package controls expose the intended work
removal without a long flowsheet campaign: selected products replace complete construction,
generated registry products replace repeated resolution, scalar cache hits avoid compiler
children, pure-ODE adjoints avoid redundant initialization, and private retained owners avoid
compatible native allocation. Their Outcomes state the actual populations and conditions.
These observations do not establish industrial sparse-fill behavior, peak working sets,
long-horizon integration accuracy or a universal native/backend winner.

No longer campaign is currently justified merely to choose among these qualified mechanisms.
I36's reopening trigger is an actual composed workload showing material unexplained factor/
setup cost, retained-memory pressure, bulk-transfer cost or a growing integration work ratio
after these changes. Start with the smallest existing authored case and vary one relevant
dimension (for example band/coupling, resident experiment count, or observation horizon),
keeping the effective method, practical accuracy and demanded results explicit. Capture the
existing phase/work/factor/integration counters and original assessment; select one detail
profile only if those counters leave a specific causal question. Keep the initial attempt
within the existing60-second cooperative budget and registered cancellation/drain route.
Choose a longer run only when that bounded comparison demonstrates which unresolved mechanism
requires it. This is a reopening route, not an authorized broad follow-up campaign.

The selected-Jacobi control also demonstrates a distinct scientific limit: a native local
error tolerance is not a global physical-error certificate, and selected First products do
not supply missing arithmetic uncertainty for generated rate providers. Neither a longer
timing run nor tighter tolerances is a substitute for that missing evidence capability.
The present controls preserve its explicit refusal and the unchanged engineering budgets.

### What was built

**Implemented and Tested, selected scope:** package Outcomes retain their actual conditions.
H02's finite inquiries are settled; H01 reconciles the completed CA01/CA02 corrections.
Independent review's publication corrections, current native populations, ordinary-stack Python worker
path and failed-product rollback pass their actual controls. Short work-removal observations
support qualified adoption without a repeated full Cartesian campaign or a universal speed
claim. The completion audit identified missing C01 boundary-work removal and E05 public
before-effects admission. Both repairs now pass their settling cases and affected checks
below; H03/H04 qualification and final closure are complete within the stated limits.

### Final qualification accounting

**Tested, selected scope; zero failure/warning baseline.** Correctness Cargo and Python
routes explicitly select Arrow validation. `scripts/pse-env` supplies the pinned checkout
and selected native capabilities. Composite repair acceptance preserves each original
failed outcome; it does not turn an earlier command into a clean pass.

| Command / mode | Actual scope and result |
| --- | --- |
| Selected canonical runtime controls | The twelve-test native selection passed 11 and failed one obsolete parent-memory injection; the corrected exact child-allowance/unchanged-output control passed 1/1. The selection includes fitting populations and exact pool balance after job drain, shooting, covariance/profile likelihood, original assessment, cancellation and real publication handoffs. |
| Selected rebuilt native Python journeys | The 21-test selection passed 20 and failed one stale settings-projection assertion; its corrected exact rerun passed 1/1. Actual ordinary-stack durable studies, strategy composition, edits, local/durable cancellation/results and relocated offline review passed. The 1,000-point sweep completed naturally in 129.66 seconds and was not repeated. |
| Current-source runtime selection | `just unit-native-package pse-runtime 'pse-runtime/native-solvers,pse-runtime/canonical-tests' 'test(math::jobs::tests::) \| test(math::strategy::target::tests::) \| test(fitting_native_foreign_population_covers_every_actual_owner_and_discards_on_cancel) \| test(authored_shooting_fixture_solves)' --success-output immediate` passed all seven actually selected tests in 0.891 seconds, zero compiler warnings/errors. This covers all five callable-target controls, including real passive-lease drop order, the actual fitting population and authored shooting. The jobs subfilter matched no tests because its actual module is `construction_tests`; it establishes no admission-test coverage. |
| Current admission/resource closure | `just unit-native-capability-package pse-runtime 'pse-runtime/native-solvers,pse-runtime/canonical-tests' solver,klu,isolation,uno,petsc 'test(math::jobs::construction_tests::)' --success-output immediate` passed 21/21 in 1.063 seconds, 1,407 unselected, zero compiler diagnostics. Actual deadline/cancellation/closed admission, population-loan, constructor/growth/pressure and exact release controls ran through the store-free route. Together with the preceding seven-test selection this is 28 actual current-source tests, not a single all-28 run. |
| Selected scientific parity | `just parity -n 0 --basetemp=/tmp/pse48-current-source-parity-pytest -k 'preflight or fit_covariance_agrees_with_parmest or sensitivity_agrees_with_ipopt_sens'` passed 7/7 in 4.35 seconds, two unselected, against the pinned Python 3.13/IDAES reference environment and current fast validated editable native extension. Five preflight checks and the actual Parmest covariance/sIPOPT sensitivity comparisons ran; no broader parity claim follows. |
| Native reset/initialization selection | Eight current reset/DAE/quadrature, native-initial-step and fresh adjoint/Second controls passed 8/8, without compiler warnings. Package Outcomes retain other mathematical, native portfolio, registry and diagnostic selections and measurements. |
| `just hygiene` and affected repairs | The continuing initial bundle completed with eight failed recipes: Ruff, Pyrefly, four generated comparisons and default/no-default Clippy. Other recipes passed, including Rustdoc, dependencies, agents, ADRs, AST/import and engine-boundary checks. Normal generator publication and the successful recipes below close those failures; the initial bundle itself remains failed. |
| `just lint-py`; `just typecheck` | Both repaired recipes passed with zero Ruff findings and zero Pyrefly errors. Existing suppressed diagnostics were not expanded. |
| `just clippy` | Full continuing workspace/all-target default and no-default modes passed with `-D warnings`, zero warnings/errors, with explicit Arrow validation. |
| Linked native continuing Clippy | Full workspace/all-targets passed with `--keep-going --no-deps --locked --features pse-py/native-solvers,pse-tests-conformance/native-acceptance,pse-benches/native-process,pse-runtime/canonical-tests,pse-runtime/compiled-evaluator -- -D warnings`, using `scripts.arrow_validation` and native compiler/solver/KLU/isolation/UNO/PETSc capabilities. This includes the applicable native contract/data lint closures. |
| Affected feature canary | Strict runtime all-target/no-default `pse-runtime/solver-diffsol` Clippy passed with `--keep-going --no-deps --locked -- -D warnings` and native KLU. An initial invocation without KLU failed to locate SuiteSparse headers; `just ready` and the corrected capability route passed. Its first check exposed four warnings, all repaired before the strict pass. The global feature powerset was not run. |
| `just governance` | Passed all 25 governance tests with zero skips/failures and all ten generated/contract/stub checks, including current schema output and family consistency. Typed optional-guard admission separately passed 4/4 actual controls. |

These results supplement the supported qualification basis; they do not claim all-case,
all-platform, industrial-capacity or release/wheel qualification. Scientific local tolerances
are not global physical-output certificates. Ordinary practical accuracy remains 0.001.
Environment/provenance proof work remains permanently eliminated.

### Completion-audit repair qualification

**Implemented and Tested, 2026-10-20–21; zero test-failure baseline.** CA01 uses selected-instance
support candidates and the existing row index at the real compiler boundary. CA02 transfers
one owned checked BBD admission before public Initial effects into native preparation.
Fresh independent source review found no material issue with the repaired consumers,
binding/teardown/stop semantics or regression coverage. This is review of these fixes,
not a second full-codebase audit.

The single current-source native-local selection ran:

```bash
just unit-native-capability-package pse-runtime \
  'pse-runtime/native-solvers,pse-runtime/canonical-tests' \
  compiler,solver,klu,isolation,uno,petsc \
  'test(assembly::ownership_tests::) | test(workspace::modeling::conditional::tests::) | test(dynamics::idas::bbd::tests::) | test(dynamics::idas::workspace_tests::) | test(kernel_bbd_prepares_value_and_exact_jvp_without_full_numeric_jacobian)' \
  -p pse-math -p pse-compiler -p pse-backend-native --success-output immediate
```

**Tested:** 27/27 passed across four binaries in 1.064 seconds; 2,896 unselected. This includes
two math ownership controls, ten compiler conditional controls, eight BBD controls, six
IDAS workspace controls and the authored runtime Value/JVP BBD journey. New tests assert
bounded actual lookup visits with 1/16/64 unrelated parent populations, complete guard/
objective/outside-row/fixed-input preservation, typed public memory refusal with zero
Initial effects and unchanged sample output, exactly-once finite admission, current stop,
configuration mismatch and failure/reset safety. The optimized test build completed in
2m 06s with no compiler warnings/errors. The existing inactive-root mode-switch control
emitted one IDAS warning about an identically zero root; both its expected native return
and changed-current-root assertions passed. This is not a zero-native-message claim.
The run used no store fixture; source/owner lifetimes drained normally.

**Tested, zero warning/error baseline:** `just clippy` passed full continuing workspace/
all-target default and no-default modes. The full linked-native/canonical/compiled mode
also passed:

```bash
scripts/pse-env --native=compiler,solver,klu,isolation,uno,petsc -- \
  .venv/bin/python -m scripts.arrow_validation clippy \
  --keep-going --no-deps --workspace --all-targets --locked \
  --features pse-py/native-solvers,pse-tests-conformance/native-acceptance,pse-benches/native-process,pse-runtime/canonical-tests,pse-runtime/compiled-evaluator \
  -- -D warnings
```

Correctness routes explicitly select Arrow validation. Rust-only body/helper repairs changed
no dependency, generator declaration, Python interface, numerical setting or enduring target
contract. Prior governance/generated/Python/parity results retain their own recorded scope;
these unchanged checks, the long sweep and global feature powerset were not repeated.
No new latency, peak-memory or universal speed benefit was measured. Actual bounded work
and public before-effect behavior, rather than a marginal timing win, close the gaps.
Documentation/current-work reconciliation closes H04 without reopening unrelated campaigns.
`just turn-end` passed ADR-index and formatting; `just docs` published 548 chapters;
`document_lifecycle validate` and `git diff --check` passed with zero errors.

### A mistake made and corrected

**Tested, actual failure and repair:** the first fast native extension build exposed a higher-ranked
`Send` obligation in initialization preparation. Transferring its prepaid request vector
into the async batch removed the redundant borrow; native Python checking and the rebuilt
editable extension/stubs pass. The initial selected Python run then exposed an incorrectly
owned shared pytest temporary directory and obsolete version-refusal message expectations.
A unique test directory preserves other owners' files. Functional receiver contexts legally
omit a primary receiver; their normal drain route now recognizes that explicit state while
still refusing an actual live unconfigured primary and preserving observer/worker/borrower
guards. Seven focused supervisor controls and normal fixture drain pass.

The subsequent selected Python run crashed in durable-study `Runtime.work()`, before a
complete JUnit result. Disassembly identifies a stack guard-page probe in the nested async
launch constructor, rather than a native solver fault. Isolating only that constructor
still overflowed: disassembly showed simultaneous multi-MiB study-start and observation
polling frames exceeding the ordinary stack before reaching it. The correction also
isolates selected study branches and claimed execution phases, erases them before wrappers,
and prepays their concrete heap extents before claim. Independent inspection and clean
native Python compilation pass. The rebuilt ordinary-stack durable study now passes,
as do the selected cancellation/result and local-study journeys. Current
activity, source permission and cancellation/drain ownership remain preserved. A separately isolated public composition test also exposed
a stale version3 gate in Python while the strategy document is version4; receiving now
infers the version from its actual typed field instead of duplicating a literal. That
public composition passes on the rebuilt extension. The selected Python run completed with
20 passes and one obsolete settings-projection assertion failure; correcting that consumer
then passes its exact isolated rerun. This is composite selected acceptance, not a second
clean whole-selection run. The 1,000-point sweep also completed naturally before a proposed
cancellation was sent; its 129.66-second call is retained as a pass and is not repeated.
The crashed tests' exact dead creator markers were archived/released
under standing maintainer permission, with database, evidence pin and incomplete outcome
preserved, followed by normal service/allocation drain.

The fitting population control separately proves immediate native coverage release.
Its shared-pool comparison must include actual accepted constructor charges belonging to
the enclosing active job. The revised control delegates those reports unchanged, requires
exact charge equality and worker capacity, retains the same immutable preparation across
job completion, and still requires the exact original pool baseline after job drain. No
byte tolerance, guessed subtraction or numerical accuracy change is introduced. This
population control passes in the final twelve-test runtime selection. That selection
passes eleven tests; its sole failure was an obsolete injection into the parent memory
allowance instead of the distinct transient owner actually performing the Hessian work.
The corrected control revokes the actual child's allowance and also requires unchanged
caller output on refusal; its isolated rerun passes, with zero compiler warnings or errors.

Scope-end checks exposed generated-byte drift from formatting and a document comment whose
owner is the schemas generator. The declared generated module now follows the existing
formatter exclusion; normal regeneration and the complete governance/codegen comparison
pass. Generated optional checks preserve short-circuit and first-error order. Sequential
outlined registry hydration and the inline width-one window owner retain narrowly explained
lint exceptions because replacing them merely for style would reintroduce compiler pressure
or an unnecessary allocation/grant. Fixture helpers propagate failures to actual test
boundaries instead of hiding them. Full native checking also exposed benchmark/conformance
profiles missing the new optimizer/method fields; those explicit native qualification cases
retain their native NLP purpose rather than silently becoming automatic-method comparisons.

Grouping current-call inputs required preserving an important early-error lifetime: passive
storage remains in the argument owner until actual admission, after callback captures on
preflight refusal. A focused real-lease regression checks that order and exact pool release.
The current-source selection also covers changed admission/deadline and shooting owners;
the final qualification table records its actual result rather than inferring it from lint.

### Deviations from the plan, deliberate

Affected feature qualification used the explicit Diffsol-only canary and full native
contract/data superset, without a global powerset or all-case campaign. Individual repaired
selections form composite acceptance, preserving their original failures. I36 selects no
currently justified long campaign, with its concrete reopening route above. Fast validated
editable installation supplies the needed Python qualification; no full distribution or
eliminated environment/provenance proof was added. Broader method/platform/contract changes
still use their concrete decision routes.

### Publication, reconciliation and cleanup

**Implemented:** enduring behavior is published with its contract owners and proposed
ADR-0201–ADR-0212; blueprint revision 238 adds the selected §24.2 qualification basis.
Plan28e, Plan40e and current-work navigation were reconciled without restarting their
independent campaigns; Plan32 remains separately paused. The earlier closeout removed
Plan48 from current-work selection; the audit briefly restored it for CA01/CA02. Their
repairs and H01/H03/H04 obligations are now complete, so completed-work selection is restored. Lifecycle preflight retains the highest coordinator and consumed
companions/reviews rather than breaking their actual consumers. Raw campaign artifacts are
not deleted by document closeout.

**Tested, ownership:** the final native/parity fixture quiesced and stopped through its
registered route, with inactive service, empty cgroup and released outer allocation. The
additional 21 native-local tests used no fixture. All owned test processes and allocations
are drained; no worktrees were created. `just docs` passed, publishing 548 chapters and their scoped search. Lifecycle validation
and read-only retirement preflight passed; consumed completed documents are retained.
`just turn-end` passed both ADR-index and formatting against zero failures. This is
formatting/index acceptance, not new product qualification.
