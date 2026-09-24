---
title: Integrated native performance I16-I19 execution
status: in-progress
date: 2026-09-20
adrs: [ADR-0074]
phase: 1
evidence: Implemented I16-I17; Tested 332 targeted units after I18 corrections; final qualification open
---

# I16-I19 execution

The approved execution order is I16 assurance/tooling, I17 complete implementation
and deletion, I18 functional assessment, then I19 performance and acceptance.
[Plan 11](11-integrated-native-performance.md) remains the scope authority.
Targeted isolated units suffice for all implementation and deletion work. Full
integration, fixture publication, solver journeys and measurements wait for I17.
Existing campaign controls remain; no additional enforcement system is introduced.

## Work remaining

- I18: refresh the editable extension, run the complete functional inventory without
  fail-fast or retries, preserve findings, repair shared causes and resume affected
  gates with authenticated unchanged evidence.
- I19: measure cold/warm/changed/evicted/contended workloads with validation modes
  separated; report timings, native counts, allocations/retention, pool peaks and
  RSS; independently assess G1-G7 and finish the decision/outcome evidence.

## Selected native interfaces

DataFusion 55.1.0 native errors, plan visitors and metrics remain authoritative.
Extend the existing projection because `find_root` only follows source chains and
`iter` only expands collections reached directly. Use native `MetricBuilder` and
`PeakRecordingPool` where applicable; retain domain counters at existing owner seams.
Arrow 59.3.0 schemas/builders/validation and declared semantic fields construct
fixtures. Delta uses the pinned source overlay, native providers, operation metrics,
commit hooks and the caller's Delta-compatible session. Native tracing previews stay
off; row caps do not establish byte bounds, and request completion is not consumed
payload completion.

## Verification

Baseline: zero failures. Environment startup: `just doctor` passed. Implementation,
functional qualification and measurements are tracked separately below; no campaign
or acceptance result is claimed by this checkpoint.

## Implementation progress

Implemented: typed diagnostic projection, including Arrow/native source traversal and
platform aggregates crossing `External`; Contract-default functional helpers, preview
suppression, explicit capture coverage and counters at existing assembly/admission,
requirement, producer, scalar and object-store seams. The fixture generator emits
1,598 separate valid/violating test functions and compares its formatted output.

Implemented: assessment dependencies, native/Python collection inventories, report
count/identity reconciliation, interruption records and authenticated continuation;
functional and measurement scopes are separate. Audit receipts retain actual argv,
exit and output. Cargo-deny findings require its documented check bitmask to agree
with the JSON summary. Workspace geiger enumeration uses real member manifests.
R-20 remains unsupported, and advisory tool errors remain failures.

Dependency audit triage removed unused declarations and an orphaned test file for
the deleted custom scan. Macro consumers (`miette` diagnostic projections and
`serde` closed enums) and externally included fixture consumers remain. The initial
removal of material's serde declaration was corrected after compilation identified
the macro expansion dependency. Detailed audit output is preserved under
`build/plan11/i16-*.log` and `build/audits/`; these are not final acceptance receipts.

Authored, not run: scalar callback timing/allocation/count characterization;
force-validation and production-equivalent benchmark/engineering recipes with
separate target directories. Engineering measurements now distinguish planning and
execution pool observation peaks from cumulative pool/RSS values.

Tested at this checkpoint: `just setup-test`, 41 isolated setup/runner tests;
`just unit-package pse-numerics 'test(pivot_unit::scalar_slots) or test(error::integrated_performance_unit::)'`,
2 selected Rust units with force-validation. Both have zero failures against zero.
The real nextest enumeration/JUnit runner probe passed 16/16 testkit observation
units in `build/plan11/i16-observation-reporter-r2/`. `just clippy` passed both normal
workspace modes in `build/assessment/20260920T093206.186280Z/`. `just quality` passed
all 15 checks in `build/assessment/20260920T093521.661893Z/` (zero baseline).
The selected Python implementation units passed 66/66; 10 component/integration
cases were intentionally deselected. Their receipt is
`build/plan11/i17-python-unit.xml`. Final source checks remain in progress.

The I17 fixture audit found and corrected the outstanding enum discriminator defect:
the generator now reads the discriminator's semantic field instead of emitting plain
text for every tagged union. A pure construction unit decodes every generated pair
without executing invariant queries. The strict Python engineering measurement
consumer now declares all native counter and phase-peak fields; these are measured
outputs, not an alternative model authority.

The acceptance manifest now carries I07-I16 and Python/tooling unit groups alongside
V01-V16 consumer-binary and gate routes. The original 179 Plan 10 obligations remain.
Independent invariant outcomes, the real threaded Python reader and actual Delta
commit/reopen and solver journeys remain scheduled for I18.

## Outcome

I16, I17 and L18 are complete at the implementation boundary. The final consolidated
selection passed 226 native units, 66 Python units and 24 runner/audit units (316
units across 74 registered groups), with zero failures against zero. Exact source
and command receipts are `build/plan11/development-checks.json` and
`build/plan11/i17-development-native-final/`. The full functional campaign (75 gates)
and subsequent measurement phase (46 gates, including both Python extension builds)
remain unqualified until I18/I19 run.

The caller audit also replaced a stale DDL-root assertion with native command/effect
inspection; primary-key/default-value assertions remain on the resulting providers.
The initial 209/211 run and corrected follow-ups are retained. Dependency removal
and final graph deltas are in `build/plan11/i17-dependency-delta.json`.
No historical campaign, advisory or deferred R-20 result is represented as passing.

### I18 correction checkpoint

The first campaign, `build/plan11/i18-functional-20260920/`, was deliberately
interrupted during `unsafe-surface`: cargo-geiger's clean/check implementation was
using the shared Cargo target. Earlier completed gates and the interruption receipt
are retained. The tool now receives a dedicated `build/audits/geiger-target`, with
an isolated unit checking that an inherited shared target cannot reach the command.
The source tree was preserved; deleted build artifacts require regeneration.

Python correctness builds now explicitly enable the workspace's Arrow validation
feature through a validation-only `pse-py` feature. I19 builds the Python extension
once in each release measurement mode, before that mode's engineering cases. Actual
feature lists distinguish force validation from production; a dev extension must
not be attributed to the production measurements. Three dependencies used only by
tests move to dev-dependencies. These corrections require a new source seal and a
linked continuation, with affected checks explicitly invalidated. The unused direct
Arrow declaration in conformance tests is also removed; those tests import Arrow
through DataFusion. The unsafe audit uses the existing pinned solver environment
because its all-feature scan includes the linked solver binding.

Tested: the corrected source passes 226 native units in
`build/plan11/i18-repair-development-r2/`, 66 Python units against the rebuilt explicit
force-validation extension in `build/plan11/i18-repair-python-final.xml`, and 25
runner/audit units in `build/plan11/i18-repair-runner-final.log`: 317 total, zero failures
against zero. `just quality` passed 15/15 checks in
`build/assessment/20260920T101722.936576Z/`; `just family-check` and actual compiled API
stub verification passed. A lock-refresh enumeration failure is retained in
`build/plan11/i18-repair-development-final/`; it executed no tests and the corrected
`r2` run qualifies the resolved graph. The [acceptance review](../design_review/reviews/design_review_integrated-native-performance-acceptance_2026-09-20.md)
keeps every gate unresolved until I18/I19 evidence is complete.

The first continuation command exposed shell splitting of a quoted change reason.
The existing assessment recipe now forwards positional arguments, with a disposable
test covering spaces, quotes and shell metacharacters. The corrected runner selection
passes 26 units in `build/plan11/i18-repair-runner-r3.log`, bringing the consolidated
total to 318. The native receipt binds Rust/manifests/configuration inputs; Python
runner scripts and the justfile are bound by their own executed unit receipt. This
keeps an unrelated argument-transport correction from invalidating unchanged native
unit results. Original captures and failed command logs are preserved.

### I18 shared-cause repairs

The `i18-functional-r3-20260920` continuation retains 42 passed gates, one failed
environment check, four advisory findings, R-20 unsupported, one interrupted native
test gate and 26 not-run gates. The native selection enumerated 2,762 tests: 2,690
passed, 37 failed/terminated and 35 did not run. It was deliberately interrupted
after repeated engineering timeouts exposed shared planning work; those results
remain historical failures. The unsafe audit completed all 33 supported package
invocations successfully. A generated proptest regression seed is preserved.

Implemented repairs bound optional field memoization so required traversals retain
pool headroom, distinguish fresh execution from actual nondeterministic expressions,
reuse closed-plan admission at shared cache ancestors, and memoize Delta effect
discovery by retained native owners. Deferred CREATE completion follows the existing
typed operation. Selected decoded Delta views preserve their declared root metadata
through a native projection at the provider boundary, with residency both enabled
and disabled. Canonical JSON generation sorts object keys independently of Cargo's
serde_json feature unification. Typed diagnostics, declared numerical literals and
explicit Diagnostic observation requests replace stale fixture assumptions.

Tested: `just unit-package` selected eight engine controls and two compiler/Delta
reproducers, all passed with force-validation against zero failures. The refreshed
source-captured development run, `build/plan11/i18-repair-r5-development/`, passed
229 native units. The runner selection passed 26 tests, and `just clippy` passed
both default and no-default workspace modes in
`build/assessment/20260920T120235.827267Z/`. These are repair/development results;
the affected integration workflows and I18 remain unqualified.

Authored, not measured: ownership/FFI retention, observation/concurrency, guarded
batch and scalar callback fixtures; separate solver, ambiguous-publication recovery
and rule-change measurement recipes. The performance inventory currently contains
76 gates, including five repetitions of each solver/recovery/rule workload in both
modes and the existing repeated engineering cases. Actual execution, feature-graph
verification, remaining model-preparation coverage and final G1-G7 closure remain I19
work. No performance result follows from compilation alone.

The r5 consolidated development receipt now covers 321 passing tests in all 74
registered groups: 229 native force-validation units, 66 Python units against the
refreshed force-validation extension and 26 runner units. `just quality` passed all
15 gates in `build/assessment/20260920T121124.485639Z/`; `just codegen-contracts`
regenerated all three contract targets without changing captured source bytes.
Generation updated Rust file timestamps, so the uv extension refresh follows it
before doctor and integration qualification. Historical doctor failures remain.

Doctor passes after that refresh. The first exact repro selection failed during
enumeration because `just test` interpolated its filter into shell source; no tests
executed. Positional argument forwarding fixes the existing recipe. Its disposable
argument-identity control brings the runner selection to 27 passing units and the
development total to 322. Unchanged Rust unit inputs retain the r5 native receipt;
the runner receipt binds the changed justfile and Python transport test.

The r6 integration repro preserves 29 passes, one ownership assertion failure,
three deliberately interrupted engineering cases and four not-run cases. Live
debugger stacks in `build/plan11/i18-r6-conservation-stack-deep.txt` identified
repeated cache sealing through shared open producers. Sealing now uses the existing
scoped native-owner rewrite and preserves already qualified owners; open producers
remain unqualified. A 24-level shared-open-producer unit checks exactly 24 owners,
no admitted proof and complete reservation release. Ownership refusal assertions
now use typed resource/cancellation codes while retaining the quota and final-reader
checks. `just clippy` passed both modes in
`build/assessment/20260920T122526.484339Z/`.

The r7 development refresh covers 230 native units, 66 Python units and 27 runner
units (323 across 74 groups), with zero failures against zero. Doctor passes after
the editable extension refresh. I18 remains open pending the next integration
repro and full functional continuation; these repairs do not close I19.

The r7 integration follow-up preserves 30 passes, two engineering timeouts at 360
seconds, three interrupted cases and two not-run cases. The conservation fixture
now accepts the normal `RUST_LOG` override for its existing subscriber. A separate
197.849-second diagnostic run was interrupted after recording 4,165,271 scoped node
visits and 533,769 field derivations; see
`build/plan11/i18-r8-traversal-summary.json`. These are repeated event counts with
diagnostic observation enabled, not a production benchmark or distinct graph size.

Implemented: structural field witnesses no longer apply the mutable-row/observation
exclusions used for completed-result reuse. Actual provider/schema, implementation,
registry and context checks remain; result retention keeps its existing stricter
test. Four focused native owner/reuse controls pass, including unchanged structural
owner identity and continued result-retention refusal for a native mutable provider.
The r10 source-captured native selection passes 230 units; both Clippy modes pass in
`build/assessment/20260920T124224.703032Z/`. The effect on the remaining engineering
timeouts is not yet qualified.

The r10 diagnostic follow-up still showed repeated large traversals. Structural
context matching also no longer depends on unrelated bindings captured by an opaque
operation's value witness: the structural proof checks its exact consumed providers
and schemas, plus the retained registry/functions/planning/policy context. A native
operation control proves stable field ownership across an unrelated binding addition,
continued refusal without the consumed source and continued refusal of value retention
for observed input. Provider inventory is captured once per sealing/rebinding pass.
The r11 source-captured selection passes 230 native units, and both Clippy modes pass
in `build/assessment/20260920T124930.572806Z/`. These controls qualify the witness
distinction; the engineering workflow and I18/I19 remain open.


The r12 diagnostic run recorded zero context/source-proof mismatches and zero open
proofs. Its stack sample instead identifies `plan_workspace` independently discovering
sources for each sibling input, repeatedly walking a 12,250-node shared graph. The
interrupted trace and stack are retained in `build/plan11/i18-r12-conservation-trace.log`
and `build/plan11/i18-r12-conservation-stack.txt`; their event counts are diagnostic
observations, not a performance baseline.

Implemented: workspace source discovery now builds one existing scope-qualified
native graph, resolves each scan against the actual binding inventory, and propagates
separate source sets to its roots. Named roles, expanded aliases, worktable scope,
empty inputs, hidden producers, cancellation and memory admission remain live. The
new 32-sibling/24-level unit preserves named-role and empty-root distinctions. The r13
compile failure exposed a native diagnostic conversion mismatch; r14 corrects it.
Tested: r14 passes 232 native units and both normal Clippy modes
(`build/assessment/20260920T130438.919470Z/`), zero failures against zero. Engineering
functional outcomes still require the subsequent repro.


The r14 trace still showed repeated large walks before shared field admission.
Implemented: sibling rebinding, finite-input admission and cache sealing now use
one native scope traversal per set. Each caller `CacheFactory::create` still receives
its input, every returned computation is admitted, and distinct outputs remain
ordered. Absent selected-source witnesses avoid an irrelevant recursive plan equality
check; present witnesses retain the original exact comparison. The factory regression
checks one call per input and cancellation before another call. The existing immutable
owner control now checks 32 unchanged sibling roots.

Tested: r16 passes 232 native units, 66 Python units and 27 runner units (325 across
75 registered groups), with zero failures against zero. `just clippy` passes both
modes in `build/assessment/20260920T131210.824622Z/`; doctor and source preflight pass
after the extension refresh. The corrected r15 compile errors and diagnostic traces
remain preserved. I18 functional and I19 measurement acceptance remain open.


The r16 diagnostic stack then identified independent compilation of required
invariants, each admitting the same combined producer graph. Implemented: missing
registry and native row checks now share query binding and field admission while
retaining individual check IDs, dependency selections and completion keys. A mixed
query/native-check unit compares separate findings, enforces the bound violation
refusal and rejects an undeclared requirement. Its initial report-style assertion
was corrected: a bound required check is a zero-violation contract.

That typed assertion exposed a separate I16 defect: completion errors entered native
`External` directly, so generic source traversal could discard their declared code.
They now use the existing typed diagnostic attachment; terminal kind, original source
and known Delta settlement evidence remain preserved. The r19 run passes the mixed
invariant and completion-code controls but reveals one stale settlement test that
expected the concrete completion at the outer `External` layer. Its exact known
version and original-source assertions are retained while updating that traversal.
I18 and I19 remain unqualified.


Tested: r20 passes all 232 native units and both Clippy modes
(`build/assessment/20260920T132444.610313Z/`), zero failures against zero. The mixed
invariant control and all completion outcome codes pass. The settlement control
uses the diagnostic projection to recover the actual completion cause and still
checks committed version 9 and the underlying native error. No required workflow
or measurement is represented as passing by these isolated results.


The r20 and r21 focused integration follow-ups each passed 30 cases. Each was
interrupted for diagnosis, leaving three terminated cases and four not run; r21's
receipt records the interrupted gate explicitly. These are not completed campaigns.
The r21 development receipt passed 233 native, 66 Python and 27 runner units (326
across 75 groups), with zero failures. Per-root freshness and obligation binding
now share one scoped traversal, preserving separate qualification/effect facts.
Only the actual native cache factory records successful intrinsic repeatability;
input reconstruction clears it. Changed-input and sibling-qualification controls
are included in the unit selection.

A further r21 debugger sample traced repeated native equality through
`Cache::with_exprs_and_inputs` during postcondition composition. Exact immutable
extension-owner shortcuts now avoid recursive comparison of the same object.
Native cache hashes select buckets by actual producer identity and planning mode;
full non-leaf equality still compares inputs, so collisions do not authorize reuse.
A 28-level shared-graph control checks equality, hashing, unchanged reconstruction
and changed-input identity. r23 passes 234 native units and both Clippy modes
(`build/assessment/20260920T135503.615143Z/`). The r22 compile failures were two
fixture typing errors, corrected before that run. The full quality run passed 14
of 15 checks; its one long Python line was corrected and `just lint-py` and
`just fmt-py-check` pass. The updated extension/combined receipt is still pending.

The remaining I19 fixtures are authored, not measured: ordinary model preparation
for cold/repeat/relevant/unrelated source changes, changed UDF/policy and retained
old revisions, over shared diamond graphs with an independent duplicate-sensitive
sum oracle. Preparation, physical planning/stream start and draining are separate;
fixed-size numeric tracing reports repeated node visits without formatting plans.
Pool reservations and cumulative process RSS are labelled rather than presented as
exact retained plan-heap bytes. Native cache workloads have five fresh repetitions
(one for smoke). Existing engineering measurements include two actual Arrow reader
tasks and independent Python work, with task times and progress counts. The same
control strengthens the functional reader test; it does not infer attachment at a
particular native call. No measurement gate or new enforcement layer was added.

Tested: the r24 combined receipt covers 234 native force-validation units, 66
Python units against the refreshed force-validation extension and 27 unchanged
runner units, 327 across 75 groups, with zero failures. `just clippy` passes both
modes in `build/assessment/20260920T135833.876277Z/`; `just doctor` passes after
`just py-sync`. `build/plan11/development-checks.json` records exact source and
command evidence. I18 engineering qualification and I19 execution remain open.

The r24 diagnostic follow-up again records 30 passes, three terminated engineering
cases and four not run. The deeper sample located the remaining comparison in
cache sealing: new wrappers around the same producer still recursively compared
all descendants. Cache equality now uses its private actual computation identity,
planning mode and complete output schema; ordering and hashing agree. Public input
reconstruction renews identity when the actual computation changes. Output-schema
mismatch and distinct producer owners remain negative controls.

The DataFusion 55.1.0 `TableScan` implementation deliberately excludes `source`
from native equality. Input matching now additionally checks actual provider and
function owners, typed operation definitions/attempts, dependencies and subquery
owners. Unknown reconstructed extensions conservatively require a new producer.
This corrects a pre-existing native-equality assumption needed by the private
identity guarantee; no canonical identity format changed. A unit proves that two
native-equal scans with different providers do not match, while new adapters around
the same provider do. Another control distinguishes same-name UDF implementations.

Tested: r25 passes 235 native force-validation units, 66 Python units after the
editable refresh, and 27 runner units, 328 across 75 groups, with zero failures.
Both `just clippy` modes pass in `build/assessment/20260920T140923.072442Z/` and
`just doctor` passes. The updated development receipt is source-qualified; the
engineering integration result and final I18/I19 acceptance remain open.


### Unchanged structural producers and changed value selection

The r25 diagnostic rerun records 30 passes, three engineering timeouts, two
interrupted engineering cases and two not run. Its receipt remains interrupted,
not successful. The r26 single-case trace identifies repeated invalidation of
producer admission after value selection changes, alongside saturation of the
optional 1 MiB field memoization allowance. That allowance is unchanged here.

Implemented: native cache reconstruction preserves structural admission and
repeatability only when actual provider/function owners and complete input fields
prove the computation unchanged. Rebinding invalidates stale completed-value
selection separately from valid structural field evidence; it still traverses
children to clear their stale selections. Scoped rewriting presents the original
owner to callbacks when native reconstruction reports no child change. Changed
inputs, foreign sources and changed semantic contexts still invalidate proofs.
No additional campaign controls or dependency changes were introduced.

Tested: r27 passes 236 native force-validation units (223 library plus 13 xtask),
including `changed_value_selection_keeps_unchanged_structural_proof` and the
existing changed-input/foreign-provider controls. Python and runner receipts,
source sealing and the engineering rerun are updated with this revision; full
I18 functional and I19 performance acceptance remain open.


The r27 diagnostic continued to revisit large ancestor graphs, despite preserving
valid structural proofs. Compiler queries were bound before finite arguments were
retained, leaving the query branch attached to original producer graphs. Arguments
now enter the native cache factory before precondition and query binding; composed
finite inputs retain their existing additional admission after transformation.
The `native_requirements_gate_actual_arguments_and_results` unit now checks that
the finite argument and composed query share an actual retained producer. Its
independent duplicate-input, duplicate-output and successful-result cases remain.
No operator executes while constructing these plans.

Tested: r28 has 236 passing native force-validation units and both `just clippy`
modes pass (`build/assessment/20260920T143521.271908Z/`), baseline zero. The r27
single engineering diagnostic was interrupted after sampling and does not qualify
its result. I18/I19 acceptance remains open pending current-source execution.


The r28 engineering sample reached P6 query composition, where DataFusion's
`LogicalPlanBuilder::normalize` recursively enumerated ancestor `USING` columns
for an unqualified generated reference. The compiler's explicit-schema projection
and filter helpers now call native
`normalize_col_with_schemas_and_ambiguity_check` on the actual input schema.
`project_with_validation` receives those already resolved expressions and still
checks duplicate output names and constructs native typed fields; filters retain
`Filter::try_new`. SQL's ancestor fallback is not needed for these generated
explicit-scope expressions. No query execution or schema check is bypassed.

Tested: r30 passes 237 native force-validation units, including a 28-level shared
ancestor regression with missing-column/duplicate-name controls; the existing
same-name role ambiguity control also passes. Both `just clippy` modes pass at
`build/assessment/20260920T144055.207236Z/`, baseline zero. The initial r29 unit
compile error (an arithmetic trait method in the fixture) was corrected to the
ordinary addition operator. r28 was interrupted for this shared-cause repair;
engineering and final functional/performance qualification remain open.


The r30 engineering diagnostic timed out at 360.425 seconds (one selected case,
zero passes, baseline zero). P6's projection normalization no longer dominated;
the last sample was in `source_bindings_many` hashing inherited lexical scopes.
Closed producers were being revisited for ancestor subquery paths even though
their structural proof establishes no dependency on those outer bindings.

Implemented: shared traversal resets lexical context at proved closed native
cache boundaries. Only freshness analysis distinguishes effect qualification;
structural/source/observation walks retain their actual edges without treating
that qualification as another intrinsic producer. Open/correlated producers keep
their contextual scope. Two regressions check bounded visits across 18 shared
subquery levels and refusal of an unqualified producer beside a read-qualified
sibling. Existing open-producer and mutation controls remain active.

Tested: r32 passes 239 native force-validation units (226 library plus 13 xtask).
Both `just clippy` modes pass at `build/assessment/20260920T145142.174487Z/`, baseline
zero, after correcting the new fixture's default-span spelling. The I19 model
preparation fixture additionally records total/peak/net retained allocations on
the synchronous preparing thread using the already pinned allocation counter.
Its explicit scope excludes pre-existing shared graph allocations and other
threads; pool reservations and process RSS remain separate measurements.
Context7 had no matching allocation-counter reference; the pinned 0.8.1 source
supplies that interface contract. I18/I19 qualification remains open.


The r32 engineering trace completed the previously stalled source-binding traversal
and reached P8 composition. A second unqualified projection in `native_sources`
then entered DataFusion's ancestor `USING` walk. Common source/row/output projection
and filter helpers now share the schema-scoped native construction path. The two
P8 context aggregates with computed unqualified fields use the same native
schema-scoped normalization followed by `Aggregate::try_new`; the default builder's
implicit-grouping option is false, so the explicit grouping contract is preserved.
The deep shared-ancestor unit now covers aggregate field resolution as well.

Tested: r35 passes 239 native force-validation units, with both `just clippy` modes
passing at `build/assessment/20260920T150213.211172Z/`, baseline zero. The
intermediate r33 unnecessary path qualification and r34 obsolete line-count lint
expectation were removed. r32 was interrupted for this identified compiler repair;
no successful engineering execution or final I18/I19 acceptance is claimed.
