---
title: Native logical-plan hard pivot — reboot checkpoint and restart
status: abandoned
date: 2026-09-14
adrs: [ADR-0067]
phase: 1
---

# Native logical-plan hard pivot — restart

**Historical checkpoint, superseded on 2026-09-15.** Resume from
[Plan 06](06-provider-contracts-hard-pivot.md) and [STATUS.md](../../STATUS.md).
Plan 06 carries forward every unfinished Plan 05 outcome and replaces the execution
order below with the provider-contract hard pivot. The saved code and receipts
remain useful evidence; this file is no longer the restart instruction.

## Latest integration boundary — 2026-09-15

**Implemented — cached configuration and deletion:** P3 and selected-method
configuration borrow the original cached AST by exact submodel key and binding
ordinal. Child `parent.*` string matching is deleted; exact i64/u64 scalar decoding
remains. The duplicate `StageContext` registry/source/policy payload, tagged-Cell
serialization and forecast are deleted. Stage index version 4 refers to actual
admitted inputs/outputs and attempt records. Reuse compares admitted invocation
and dependency owners, including source and policy graphs.

**Tested:** compiler/authoring pass 82 tests; full native normalization passes
6 tests, including two-parent configuration with exact u64 values. The later
nine-package foundation suite passes **568 tests, 0 failed/skipped**, 14.899 s,
default/force-validation/baseline 0, run `f23e6746-975a-406c-ac31-37f9f1e736ca`.
Exact commands and conditions are in Plan 05's latest integration subsection.
The subsequent catalog/reuse suite passes 30 tests; the bounded comparison and
actual port-owner suite passes 9 tests, with 0 failures (ci/force-validation/baseline 0).
Dependency traversal now reserves and cancels through the request resources.
The obsolete bare-stage fixture is deleted after moving its assertions to the actual
registered producer. The full engineering rerun passes **4 tests, 0 failed/skipped**,
593.216 s, ci/force-validation/baseline 0, run `10c8df55-bca4-4a03-8b6f-072c68c8dbef`.
It precedes the following producer/input cleanup. HP00–HP13 remain incomplete.

**Implemented and Tested — common producer result:** `PassOutput`, `PassRecordDraft`
and duplicate output validation are deleted. The catalog owns completed production;
the Driver owns terminal attempt status. Completion/terminal tests pass **13 tests,
0 failed/skipped**, 103.378 s, ci/force-validation/baseline 0. Driver and cold producer
restoration now bind actual named input roles; `InputBundle::rows` is deleted. The
named-input suite now passes **11 tests, 0 failed/skipped**, 72.737 s,
ci/force-validation/baseline 0, after rebuilding the lost executable. It exercises
before/after role execution through the Driver and cold reopen. Policy selection now
reads the admitted Arrow identity column directly; generic Cell decoding is deleted.
`just quality` passes with 0 findings. Exact commands and source boundaries are in
Plan 05. The full foundation rerun passes **568 tests, 0 failed/skipped**,
15.122 s, default/force-validation/baseline 0, including the policy-key deletion and
version/empty-role provider tests. The editable Python extension is being refreshed.


**Implementation remains incomplete.** All four heater/mixer FTPx/FcTP workflows
now pass from source through P10: **4 passed, 0 failed/skipped**, 599.359 s,
ci/force-validation/baseline 0, run `becb477b-1262-4c5c-82cc-81bd527c7d45`.
Command: `just test-package pse-tests-engine --test native_engineering_workflows
--profile ci --no-fail-fast --status-level fail --final-status-level fail`.
This exercises their independent graph expectations. Full cold reopening,
Python, broader scientific/provenance and terminal acceptance remain open.

**Implemented and Tested:** law/connection zero bounds, state expressions and mixer
pressure equality explicitly broadcast their missing axes. The physical package
declares the required absolute-pressure difference over inlet binders. Actual
inference tests reject incompatible binders and missing broadcasts. `TargetContext`
and its nine-family decoder are deleted. P3 reads checked Arrow columns; lexical
binding retains one instance inventory and rename reads entity ancestry. P7/P9 now
use one realization entry point with the actual pass context and selected inputs.

**Tested:** the nine-package foundation suite passes **567 tests, 0 failed/skipped**,
14.926 s, default/force-validation/baseline 0, run
`c5ff2ea8-f3f4-4a2b-a700-6ede82f27df3`. Exact command is in [STATUS.md](../../STATUS.md).
It covers the realization entry point, indexed-pressure difference, generated adapters,
P3/P10/native-construction helpers, borrowed operands and symbol-ID-only support.
The latest `just clippy` run reaches 71 compiler library findings (72 with library
tests), after clearing rules and authoring. The foundation receipt includes the admission-binding pivot; the Clippy receipt predates the document-source follow-up.

**Tested — Python and environment:** `just py-sync` rebuilt the editable dev extension
and generated stubs from its actual API. `just py-test -n 0` passes **80 tests,
0 failed**, 48.83 s, Python 3.14.7, unit/component mode, baseline 0; 55 parity tests
are deliberately deselected. The recipe first publishes/admit two fresh current
snapshots. `just check` reports zero warnings/errors across all workspace targets.
`just doctor` reports matching environment/extension lockfiles and Environment ready.

**Implemented — durable invocation bindings:** the manifest now selects one immutable
checksum-addressed binding containing exact parent encodings, the selected original-document source, policy owners
and sealed engine settings. Publication, explicit-context admission, pinned reopening
and durable reuse consume that binding. The target-keyed receipt and default-policy
replay fallback are deleted. Native engine restoration retains the supplied actual
implementations, restores explicit absent settings and refuses incompatible context.
Invocation/configuration allocations retain their own reservations. The copied stage
context has since been deleted; wider HP06 lifecycle/implementation-binding gates remain open.

**Tested:** the five-binary catalog/normalization integration command in STATUS.md
passes **31 tests, 0 failed/skipped**, 244.627 s, ci/force-validation/baseline 0,
run `9bc1efe9-9ae0-408d-adee-77e1eedd145e`. Traversal now retains the decoded manifest
instead of reading and decoding it twice. The extended Driver restart-reuse and ownership checks pass in a subsequent
`stage_ports` run: **3 passed, 0 failed/skipped**, 17.584 s, ci/force-validation,
run `055409a1-f471-445b-99d9-76dff762e66b`. New durable record bytes retain their
reservation until the in-memory store and all its snapshot owners are dropped.

**Implemented — deletion:** the row migration interpreter, automatic version-path
search and generated migration lookup are deleted. Explicit native schema transforms
reuse actual edit declarations as projections and checked UDFs, with residual relation
obligations. External-stage producer checking now lives in the catalog's common
registered execution path. The overridable stage-validation callback and separate
compiler comparison module are deleted.

**Tested:** catalog integration tests now live in the engine test family and execute
the actual invariant validator and explicit native producers. All **26 pass**,
15.376 s, default/force-validation/baseline 0, run
`1d3b9010-4d2a-4619-8160-e69cd871c51b`. This includes source/producer refusal,
forged parent keys, immutable receipt and stream ownership, cancellation and codec
bindings. Hand-written change-set declarations were replaced by their canonical
registry declarations. `just quality` passes with 0 findings/errors and all 14 setup
guard tests.

The driver shares actual invocation owners once per request engine mode. The
complete context-copy path has been deleted. See the latest subsection above for
current verification; earlier receipts do not close HP00–HP13. Generated builder
lint causes were corrected at the generator and regenerated, but workspace Clippy
has not yet passed. Governance passed 59 tests before failing generated-file
tracking hygiene; preserve the shared index.

All legacy `tests/golden` stores and their generator/callers are deleted. Python
inspection fixtures construct current stores through the Driver. The editable
extension was refreshed and 80 ordinary Python tests passed before the latest
configuration and stage-index changes. Refresh it again after native integration
stabilizes. Full scientific/provenance, cold P10/Python and terminal acceptance remain open.

## Current continuation — 2026-09-15

**Maintainer clarification:** this remains a design-phase hard pivot. Retain graph
inputs only for an actual target computation or reuse. A current P7 result used to
construct P8 laws is such an input; historical graphs, compatibility copies and
predecessor-equivalence requirements are not. Delete legacy code and data objects
as their target replacements land.

**Implemented, incomplete:** P3 main now uses actual admitted inputs, immutable
documents, checked configuration and typed source lowering. `p3/source.rs` and its
support module retain expression/configuration/unit origins; `native_outputs` joins
each occurrence by source relation, port and exact key. `p3/seeds.rs` retains only
syntax traversal. Its native planner selects property mappings and expands finite
opaque indices with actual domain/member sources. Shared native construction and ordered-axis helpers now live
in `native_construction.rs`; the separate `port_paths/plans.rs` is deleted. Source lowering
supplies explicit empty predicate/index/path inputs before products consume them.
Full P3 workflow and scientific/provenance acceptance are still pending, including
selected-method configuration and source-binding dependency completeness.

**Implemented:** `with_metadata` carries declared field annotations in native
projections after semantic compatibility checks. `pse_preserve_field` is the shared
checked-identity UDF for nested scalar field recovery, inserted visibly after native
constant folding. It preserves established input meaning and rejects layout/meaning
changes. `AdmittedField` and its physical-expression implementation are deleted;
the narrower Cartesian implementation selection remains. Completed materialization
caches the actual checked result and retains existing buffer claims. Initial local
value admission and broader property/reservation transfer remain open.

**Implemented, incomplete:** P4 predicates and coordinates use typed native
arguments; P5 ports, paths and tears use native construction or bounded typed
algorithms. `p4/augmented.rs` shares those implementations with P9 and preserves both
completed rule families in a native evidence union. P6 now uses native requirement,
scope, dependency and parameter-coordinate plans plus the shared coordinate evaluator.
The predicate/structural/P6 replay constructors and generic structural row helper are
deleted. The unused P5 traversal relation and unlinked duplicate port implementation
are also deleted. None of these compiler consumers has behavioral acceptance yet.

**Implemented, incomplete:** P7 now appends generated Arrow columns through
`native_outputs::OutputRows`; typed dependent views keep the actual output owner.
`p7/outputs.rs` couples emitted graph columns to node/equation/kernel source evidence.
`p7/evidence.rs` selects typed arguments and actual projected keys; the whole-input
Cell mirror and manual derivation builder are deleted. Selected realization loads
the current mathematical graph from its declared inputs, remaps root columns in a
native plan and retains original source keys in that same completion. The old public
map-based `realize_selected` helper is deleted; private shared realization now also
requires actual `Sources`. Coordinate evaluation and realization share one source
graph with its allocation lease. Full transfer and method workflows remain untested.

**Implemented, incomplete — 2026-09-15:** P8 uses generated typed law arguments
and native source keys. Current graph/root projection uses checked native plans;
the generic row copy/remap and broad manual derivation fan-out are deleted. Domain,
fixed-subject, subgroup, connection and coefficient outputs use generated columns.
The coefficient join carries actual member, eligibility, species and composition
keys in the same completion. Old `ConstructedInput`/`WorkspaceConstructor` code and
its mapping module are deleted; the native support mapping remains active.

**Implemented, incomplete:** P9 parameter/method/kernel adapters use checked native
arguments and generated outputs. The coordinate projector selects actual mappings,
domains, members and material pairs in one native completion. P7 extends only the
current graph used by selected methods; no historical graph is retained. The old
instance comparator, generic output/provenance helpers and unused replay-error
wrapper are deleted. Pass declarations no longer forward temporary P8 rule facts
as mathematical graph outputs. The P10 fixture graph/importer and golden subworkflow
are deleted. Source tests use independent physical declarations in
`tests/fixtures/packages/physical-primitives` and ordinary Driver execution.

**Interface-checked:** `just check-library pse-compiler` passed with 0 errors and
0 warnings, locked/dev/library-only, baseline 0. **Tested:**
`just test-package pse-compiler --lib --test math_families --no-fail-fast
--status-level fail --final-status-level fail` passed **22 tests, 0 failed/skipped**,
nextest default with Arrow `force-validate`, run
`a0c6e7ec-1ecb-4f0e-bad0-14c5c4e92c7d`. Native product tests exposed and corrected
missing nullary UDF signatures and CASE annotation loss. `same_field_case` checks
actual branch fields and uses native `with_metadata`; ordered subsets use native
list slicing to keep child meaning. P3 now includes the recorded literal-unit
origins in its emitted provenance; workflow regression remains pending.

**Implemented, incomplete:** P3–P10 declare native plan execution. External predecessor
injection (`ExternalInputs`/`fixture_mode`) and its dedicated fixture test are deleted.
The old raw normalization harness and stock-package `normalize_owned` caller are
deleted; target tests now enter through ordinary source commits. Current ChangeSet
wire version 2 stores complete staged batches, preserves operation row order on
reopen, rejects invalid ordinals and requires every batch row to be referenced.
Its header/operation/revision admission uses generated Arrow views.

**Implemented and Tested:** `pse_array_agg` delegates accumulation, ordering, distinct,
grouping and partial/final state handling to native DataFusion while retaining the
actual argument field as the list child. `pse_array_element` delegates array selection
and preserves the actual child field. Both serve compiler and rule construction.
Pinned 55.1 source and probes exposed the datatype-only aggregate field and
`SingleRowListArrayBuilder::with_field` copying name/nullability without metadata.
The adapters avoid that lossy scalar reconstruction. Native derivation accepts
missing child metadata only when established by the actual expression/source;
conflicting meaning and layouts remain errors.

**Next integration boundary:** current source-to-P3/P7 engine tests and all remaining
callers. The first six-test workflow run passed the actual ChangeSet batch/reopen
test and failed five normalization/template cases. Those failures exposed missing
field propagation in array selection and a missing test package unit-set binding;
corrections are undergoing source-workflow verification.
P8/P9 graph/source completeness, allocation behavior, full regeneration, Python,
product workflows and all HP00–HP13 terminal requirements remain open.

**Tested:** the catalog metadata/output/refinement command in Plan 05's
implementation-continuation section passed 11 tests, 0 failed/skipped and 0 warnings,
nextest default and Arrow `force-validate`, run `f4cd08a6-7db0-48b2-830f-ed77ff14d044`.
This includes scalar/nested selection, negative and out-of-range indices, and ordered,
distinct and grouped aggregation across two partitions.
Full codegen, Python refresh, P0–P10 workflows and terminal gates remain open.

**Tested:** the catalog foundation command in Plan 05 reports 113 passed, 0
failed/skipped (run `5174e575-f0e6-4224-a609-682557aa3238`); the native rule command
reports 17 passed, 0 failed/skipped (run
`425ffe9c-aa34-4957-be4d-f20f544c2838`). Both use nextest default with Arrow
`force-validate` and baseline 0. The latter includes independent source-role owners
and swapped-witness refusal. Full compiler/Python/product acceptance remains open.

The following sections preserve the pre-reboot checkpoint. Where they describe P3
main as unreplaced or report 56 errors, use this continuation and current code.


**Implemented and Tested — current native rules:** rule column bindings now preserve
logical names independently from native join scopes. Assertion deduplication uses a
native window to retain original Arrow payloads, including signed zero and nested
fields. Fact derivation links select an actual ordered assertion identity and use the
shared `pse_require_nonnull` UDF; the NIL fallback is deleted. The complete rules
command `just test-package pse-rules --no-fail-fast --status-level fail
--final-status-level fail` passed **49 tests, 0 failed/skipped, 0 warnings**, nextest
default with Arrow `force-validate`, baseline 0, run
`967692fe-169f-4d8e-b112-54a69eb06dd7`.

**Tested — complete compiler package:** `just test-package pse-compiler
--no-fail-fast --status-level fail --final-status-level fail` passed **28 tests,
0 failed/skipped, 0 warnings**, nextest default with Arrow `force-validate`, baseline
0, run `91f999f0-8861-475a-abe2-c0bce1a9bf0d`. This supersedes the earlier library-only
and selected-test receipt, without claiming source-to-P10 acceptance.

**Implemented, pending workflow verification:** package unit normalization derives
its target from the explicit package base-unit set, allowing equivalent component
spellings. The source tests check resolved symbol identities and explicit conditional
demand guards. The last six-test workflow run (`b33c59c0-76ec-42c5-b9f5-7610fbfeb5d4`)
passed 3 and failed 3, with 0 skipped: the failures exposed the unit ambiguity, a
lexical-reference test expectation and P4 derivation-link metadata. Corrections are
undergoing fresh source tests. Engine admission/terminal fixtures are being migrated
to sealed actual producer registration; the empty-carrier and permissive source
projection fixtures are deleted.

**Implemented — legacy data deletion:** all 132 files in the former P10 golden
fixture store and its golden observation are deleted. Remaining golden artifacts
still require target-format regeneration; they are not an acceptance baseline.


### Native integration receipt — 2026-09-15

**Tested:** all four actual P0–P3 normalization cases passed in run
`768d4c0a-97d2-4252-af42-d2c6fa335341`; the complete six-test command had **4 passed,
2 failed, 0 skipped**. It was `just test-package pse-tests-engine --test
native_normalization --test native_template_graph --test native_reference_packages
--no-fail-fast --status-level fail --final-status-level fail`, nextest default,
Arrow `force-validate`, baseline 0. The template case completed P4 and failed in P5
on optional state identity meaning. Reference loading found an entity-name collision.

**Implemented, under fresh workflow verification:** P5 optional state targets use
checked typed NULL and the shared CASE carrier. Value meaning must agree; only common
role/FK annotations transfer. Internal method templates have distinct `.template`
qualified names; public method names are retained. The templates document declaration
now includes its existing port-member relation. Package closure recognizes an exact
version requirement with an optional `=` prefix through native `regexp_replace`.

**Tested:** `just codegen-bootstrap` completed its Rust-contract bootstrap and all
three complete schema targets, **0 errors/warnings** (bindgen remains deferred R-3).
`just fmt-check` passed Rustfmt and pinned Taplo, **0 findings**, baseline 0.
**Interface-checked:** `just test-package pse-tests-engine --no-run` compiled all
engine test targets, **0 errors/warnings**, locked with Arrow `force-validate`.

**Implemented:** engine callers use immutable source loading and sealed producer
registration. Commit requests retain `OwnedChangeSet`, including its allocation lease,
through application. Mutable parsed-document and detached rename test paths are deleted;
source mutation tests construct freshly parsed inputs. Their runtime acceptance remains
pending, including terminal fault injection and complete rename publication.

**Tested:** `just test-package pse-rules --no-fail-fast --status-level fail
--final-status-level fail` passed **50 tests, 0 failed/skipped, 0 warnings**, run
`6df7217b-d35d-40c3-9b51-b009eefcd0a7`. The added fact-link test checks the smallest
actual assertion ID and empty-input behavior. `just test-package pse-catalog --test
native_nested_output --test native_output --test filter_refinement --no-fail-fast
--status-level fail --final-status-level fail` passed **12 tests, 0 failed/skipped,
0 warnings**, run `8299221d-6d27-4147-95ac-37f4f7c6ac54`. Both use nextest default,
Arrow `force-validate`, baseline 0.


### Native field and source continuation — 2026-09-15

**Tested:** the source-projection admission case passed in run
`85fc3d66-2ab0-45fc-afc5-a478c2faf8c7`. The complete command was
`just test-package pse-tests-engine --test native_template_graph --test
native_reference_packages --test source_projection_admission --no-fail-fast
--status-level fail --final-status-level fail`: **1 passed, 2 failed, 0 skipped**,
nextest default with Arrow `force-validate`, baseline 0. The test reopens an actual
committed Model and rejects changed package values without a source edit.

**Implemented:** P5 guard defaults use an explicitly checked TruthValue dictionary
literal and shared CASE/nonnull expressions. Internal phase names and the isothermal
balance equation now have unambiguous names in shipped source declarations. Source
binding and literal-storage failures include the actual offending field/name.

**Tested:** `just test-package pse-rules --test relational_execution --no-fail-fast
--status-level fail --final-status-level fail` passed **9 tests, 0 failed/skipped,
0 warnings**, run `5ed7fe27-23ab-40d5-8391-a0f1ababc58b`, nextest default with Arrow
`force-validate`, baseline 0. Native null-safe comparisons are wrapped in native
`IS TRUE`, preserving their total Boolean result while exposing non-nullability
through pinned DataFusion 55's field machinery. The regression checks both operators
against Boolean and NULL values.

**Implemented; runtime receipt follows below:** ChangeSet construction reservations
travel with envelope clones retained by candidates. The staging wrapper no longer
holds the only allocation lease. The existing detached-source lifetime test now also
checks an independently retained envelope after all staging handles drop.
`just fmt-check` passed again with **0 findings**, baseline 0.

Current integration continues through actual source commits; full HP00–HP13
acceptance remains open.

### Native integration continuation — 2026-09-15

**Tested:** the native engine integration run
`025b4505-f6ea-415c-99d7-1510b203f4f4` completed **8 passed, 4 failed,
0 skipped** in 197.244 seconds. Command: `just test-package pse-tests-engine
--test native_template_graph --test native_reference_packages --test terminal_attempts
--test memo_dependencies --no-fail-fast --status-level fail --final-status-level fail`.
All seven terminal-attempt tests and the actual declared P3–P10 dependency-chain test
passed. Source preparation now runs within the first pass attempt, so pre-cancellation
retains its terminal record; fault tests use the generated current pass-record version.

**Tested:** the subsequent focused engine run
`f74442e7-f6bf-4615-bbe8-66e1472d7064` passed all three memo dependency tests:
**3 passed, 2 failed, 3 filtered out**, 123.000 seconds. Command: `just test-package
pse-tests-engine --test native_template_graph --test memo_dependencies --test
commit_p0_p2 -E "'test(two_templates) | test(full_rename) | binary(memo_dependencies)'"
--no-fail-fast --status-level fail --final-status-level fail`. The fixture uses the
actual invariant validator with its declared diagnostic output. Remaining failures
were the rename application's absent-empty source member and P6 dependency index
construction. Both corrections are implemented; subsequent workflow results are recorded below.

**Tested:** `just test-package pse-catalog --test native_output --no-fail-fast
--status-level fail --final-status-level fail` passed **4 tests, 0 failed/skipped**,
run `1ebf4846-18a1-4d62-9a38-d1a39c05ed29`. `just test-package pse-authoring
--test rename --no-fail-fast --status-level fail --final-status-level fail` passed
**4 tests, 0 failed/skipped**, run `bb7d277a-804d-44d4-9cd4-3ce3ae6aee39`.
These establish checked dictionary CASE values and the last-reader reservation
lifetime of staged envelopes and detached Arrow buffers.

**Implemented:** P6 maps DomainKind to explicitly constructed ScopeKind values and
uses the shared `pse_index_tuple` UDF to construct its declared index outputs from
ordered actual identities. Parameter-selected source paths accept explicit qualified
enum literals; resolved paths retain their actual leaf enum, and predicate evaluation
requires exact enum identity. No predecessor graph is introduced.

**Tested:** `just test-package pse-authoring --test load_package
enum_predicates_bind_exact_known_members_and_explicit_deferred_types --no-fail-fast
--status-level fail --final-status-level fail` passed **1 test, 0 failed, 5 filtered
out**, run `6bbb05f8-b4d5-4e7e-9915-ee51dfde515d`. It exercises seven cases: known
contextual and deferred explicit bindings, undeclared members, incompatible enums,
non-enum operands, missing context and forbidden enum ordering. The preceding
complete load-package run passed its other five tests; two fixture-declaration errors
in the new case were corrected before this receipt.

All Rust receipts above use nextest **default**, Arrow **force-validate**, failure
baseline **0**. The product workflow, full workspace and Python gates remain open.

## State to preserve

### Current integration boundary — 2026-09-15

Continue from the current configuration/source, format-3 stage storage and P6 native
rules. The complete rules suite passed 53 tests, authoring load-package passed 6,
and the stage-format test passed 1 (default nextest, Arrow force-validation,
failure baseline 0). Actual full rename and P3 reuse passed in mixed integration
runs. The current `ci` source-workflow run passed **6 tests, 0 failed/skipped**,
426.549 s: four normalization cases, shipped reference normalization and source-to-P7
template construction. The template test now extends through P10. P8 axis nullability
and MIN metadata failures are corrected. P9 now completes with its actual declared
inputs; P10's UNION metadata mismatch is corrected in source. The template/FTPx heater
run is active. The current rule suite passes 53 tests, compiler library tests pass
22 and native aggregate/field/output tests pass 13. See the new aggregate
and Python inspection continuation in Plan 05 for exact commands and run IDs.
P3's per-expression materialization branches/unions are deleted; complete relation
batches retain exact source associations through the shared builder. Physical inputs
have one selector shared by the inventory, pass ports and P7 evidence. Full graph,
Python and terminal qualification remain open.

See [the exact commands and receipts](05-native-logical-plan-hard-pivot.md#source-configuration-current-storage-and-demand-integration--2026-09-15).
P3 no longer emits unused submodel configuration graphs. Stage hints require complete
context; obsolete formats and the context-free writer are removed. Preserve these
deletions and continue the target architecture directly.

- Checkout: branch `wave2/semantic-compilation`, HEAD
  `eebc82e9f2298e4a53e08818cfd2a1dfb6fc310b` at checkpoint.
- Implementation is in the dirty working tree, including untracked source and
  generated files. No checkpoint commit, push, reset, clean, or stash was made.
  Existing policy changes and earlier-wave work also remain in that tree; the full
  diff is not attributable solely to this checkpoint.
- **Historical pre-reboot receipt, superseded above:** the compiler did not build.
  That `just check-library pse-compiler`
  returned 56 compiler errors against baseline 0, exit 101. They are concentrated
  at deliberately replaced P3–P9 APIs. Do not restore the old APIs just to compile.
- Pure Rust contract generation succeeded, including
  `provenance.algorithm_source_occurrences`, `provenance.node_rewrites`, and
  `inferred.port_binding_walk`. The earlier complete `just codegen-bootstrap` failure
  is superseded: the current Rust bootstrap and all three complete schema targets
  regenerate successfully. The generated-tree hygiene gate remains unverified in
  the substantial untracked working tree. Never hand-edit generated paths.
- The extension now exposes `EngineSettings`, and its stub is generated from actual
  PyO3 metadata. Python static checks pass. The first fresh-store inspection run was
  19 passed/1 failed on a test lookup now corrected. Refresh the extension after
  subsequent native edits and rerun current Python acceptance. Legacy stored golden
  fixtures and the comparison/publisher command are deleted; use `just py-test`.
- Agents stopped after narrow current edits. No task-owned build, test, codegen,
  or server process is intentionally left running. No background implementation
  should be assumed after restart.

## Direction that must survive restart

Use blueprint revision 37 and ADR-0067. Arrow is the default for typed data and
columnar operations; DataFusion is the default for transformation, planning and
execution. All library capabilities remain eligible; other libraries are welcome
when they offer a distinctive advantage. Native `LogicalPlan`, actual function
objects, fields, constraints and physical plans carry the computation. Do not
create a second optimizer, closed relational algebra, or proof DSL.

Validity comes from actual immutable input ownership, checked construction and the
small residual obligations that construction cannot establish. Hashes label
identity, integrity and lookups; equality of hashes does not prove validity,
uniqueness, convergence, producer correspondence or safe reuse. A logical plan is
not a universal formal proof of arbitrary scientific algorithms or I/O effects.

Continue direct replacement and deletion. No compatibility adapters, old-store
migration, dual runtime, old/new equivalence campaign, or fixture-produced graph
as the product gate. Generous configurable shared resources remain intended: the
current workflow default is 32 GiB, with independent thread/partition controls.
Tiny budgets belong in explicit refusal tests. Do not reintroduce the former
512 MiB, one-million-row or 256 MiB production limits.

The product target remains real P0–P10, current-format publication/reopen and
immutable Rust/Python inspection, including FTPx and FcTP heater/mixer workflows.
P11–P16, solving and Pyomo remain later-wave scope. Do not restart broad design
research: use the capability maps and pinned evidence to answer concrete gaps.

## Immediate restart order

1. Read `AGENTS.md`, applicable rules, Plan 05 and this checkpoint. Run
   `just doctor` and `just --list`. Preserve the source tree. Use one native build
   lane; agents may edit disjoint files but should not launch overlapping Cargo
   commands. The stale extension is expected until the native rewrite compiles.
2. Finish the P3 checked orchestration and exact source coupling described below.
   Integrate configuration and the already written products before replacing the
   remaining demand-seed relational loops. Keep P3 lowering a typed algorithm.
3. Complete shared `IndexEvaluator`, P4–P6 native consumers and P7–P9 coupled
   seed/graph/output rewrites. Rebuild all affected consumers of checked
   `PassOutput`; do not add a temporary raw-batch compatibility route.
4. Run `just check-library pse-compiler`, then `just codegen-bootstrap`. Fix current
   declarations/generators, regenerate, and move the old public fixtures/tests to
   real target contracts. Run focused package tests with `force-validate` through
   the recipes. Successful compilation alone does not close any domain package.
5. Complete property transfer, resource ownership, durable invocation context,
   publication effects and producer/observation integration. These are required
   architecture work, not optional hardening after a green compiler.
6. Complete HP08–HP11 behavior and real HP12 workflows, then HP13 deletion,
   measurements, focused G1–G7 review and terminal commands from Plan 05.

## Shared source contracts now present

**Implemented; acceptance varies by the receipts below.** These APIs replace
the old row/callback routes. Read their implementations before adding wrappers.

| Source | Current contract and remaining integration |
|---|---|
| `pse-relations/src/columnar` | Private `FieldCheckedBatch` owns exact declaration and Arrow fields; generated `RelationRow`, builders, views and `Collection` provide typed access. `admit` is raw field admission, not a PK/FK proof. `concat_reserved` reserves before concat, reuses a single input and retains owners. |
| `pse-catalog/src/session` | Native preparation/completion retains actual plan, frozen state, sources/functions and resource owners. Checked inputs can be reused without executing a scan. Physical observations and wakeable cancellation are source-linked. |
| `pse-catalog/src/session/indexed.rs` | `with_indexed_checked_role` installs immutable algorithm outputs with a constructed row ordinal before planning. `scan_computation_role` preserves that ordinal through native reorder; it is not invented from execution order. |
| `pse-rules/src/strata/native_input` | `NativeInput::build(plan, output, pass, columns, witnesses, session, cancel).await` binds actual native completion to exact source witnesses; `union`, `checked` and `derivations` retain that construction. Conditional positive witnesses and explicit complete read scopes are separate. |
| `passes/native_rows.rs` | `AlgorithmInputs` retains native completions, sessions, checked batches and typed algorithm allocations. `keyed_rows<T>` selects typed rows and exact source keys with native ordering. Keep key/row correspondence under repartitioning explicit. |
| `passes/native_outputs.rs` | `OutputRows` emits generated checked columns while capturing actual per-row `SourceKey` occurrences. `GeneratedOutputs` carries columns, occurrence batch, positive sources, read scopes and a reservation. `materialize` joins exact source keys before `NativeInput` construction. |
| `passes/native_sources.rs` | Shared pass-bound source binder, extracted from P8. `Sources::with_overrides` accepts actual retained Completed/Native sources inside the pass inventory. This binder is distinct from the `native_outputs::Sources` map; use each actual API deliberately. |
| `quantity_relations` | `PhysicalInventory::load` uses actual native physical-definition/unit joins; quantity/material/precondition consumers borrow one inventory. `RelationSymbolSource::load(ctx, inputs).await` binds the real P10 stage inputs. |
| `mathir_relations` | Generated checked `RelationSink::new(registry, family, reserver, cancel)` emits `into_batches`; `RelationSource::from_checked` and async `load_family` replace raw row constructors. `into_rows` and duplicate physical decoders were deleted. |
| `PassOutput.ports` | One `FieldCheckedBatch` per declared port, not `Vec<RecordBatch>`. The registered producer executes once; private completion binds output and context before publication. P3–P9 callers are still being converted. |

The transient occurrence relation has
`(output_relation_id, constructed_row_ordinal, source_relation_id, source_key)`.
`node_rewrites(input_node_id, output_node_id)` is scoped to one retained graph and
family role; it must not conflate unrelated node-number namespaces. Neither is a
second authority for facts. Source keys identify rows that the plan joins against
actual retained inputs; they do not independently establish source membership.

## P3 replacement boundary — HP05

**Implemented source, incomplete integration:** typed `lower.rs`,
`demand_paths.rs`, `units.rs`, `selectors.rs`, `native.rs`, `primitives.rs` and
`provenance.rs`. The last compiler check reported no errors in these new helpers;
their complete workflow has not run. `p3.rs` still calls the removed row APIs and
accounts for 24 of the 56 current compiler diagnostics.

- `lower` constructs generated predicate/equation/binding/path rows from owned
  source syntax. Children precede their parents. Enum ordering restrictions moved
  into comparison construction. The old `p3/validate.rs` retrospective interpreter
  was deleted; retain essential syntax/algorithm preconditions at construction.
- `primitives::emit` uses native projections and exact source witnesses.
  `selectors::emit` uses typed syntax folding with captured actual source keys;
  selector set evaluation remains native rule work. `SourceBindings::source_key`
  now exposes the binding's actual document/path key rather than reconstructing it.
- `provenance::Construction` owns original and completed output sources, replaces
  branches, exposes checked outputs and builds a session over those actual owners.
  `capture` binds generated row ordinals to source occurrences; `materialize`
  delegates to the shared native output implementation. The old
  `provenance/dependencies.rs` graph reconstruction was deleted.
- `seeds.rs` is only partially converted to generated rows. Native relational
  matching, source joins, index selection and Cartesian expansion still need to
  replace its procedural mapping/product loops. Do not call this native completion.
- `port_paths::emit(checked, session, cancel).await` produces checked path rows and
  derivations. Integrate its actual native producer/source ownership with P3's
  construction map; an arbitrary checked batch wrapper is not sufficient provenance.

**Configuration API already written:**

```rust
Configuration::build(&BTreeMap<SemanticId, FieldCheckedBatch>,
                     session, registry, physical, reserver, cancel).await
configuration.binding_batches()
configuration.emit(&actual_expression_sources).await
// -> Configured { batches: BTreeMap<RelationKey, FieldCheckedBatch>,
//                 arguments: AlgorithmInputs }
```

Configuration uses native assignment/default/membership joins and generated
builders. The specialized recursive instance expansion is an explicit typed
algorithm. **Remaining:** capture precise per-output row source occurrences and
finish configuration as actual NativeInputs. Retained argument owners alone do
not give that exact output provenance. `configure_selected` has the same checked,
async context and emits base configuration; shared products follows separately.
`SelectedRoot` now has a typed instance, `source_relation` and exact `source_key`.

**Products API already written:**

```rust
products::emit(&checked_inputs, &native_outputs::Sources,
               pass, session, cancel).await
// -> BTreeMap<RelationKey, Arc<NativeInput>>
```

`products/{native,shapes,ancestry,expansion}.rs` replaces the old Cell product
engine with native joins, recursive ancestry, ordered subsets/aggregation and
Cartesian tuples. It preserves repeated ordered axes, real scalar consumers,
empty and continuous factors, exact source keys and explicit cycle refusal.
Three tests in `products/native/tests.rs` are **not run**. A pinned `string_agg`
import was fixed; the latest library check reports no errors in products. Its
behavior, source membership and recursion still need execution.

**Finish P3 in this order:** bind original checked inputs and actual locations;
construct native primitives; build configuration and owned source binding; lower
each source with typed MathIR sink and exact source occurrence; construct derived
units and retain reference-unit/package-unit-set dependencies; complete source and
configuration branches; selectors/paths; native products; native demand seeds;
explicit empty ports and complete derivations. Accumulate/union multiple source
branches before replacing the construction map. Replace `source_row` with generated
`expression_sources` and nested key items, and `owner_package` with actual native
binding. Raw `normalize` may admit its external inputs once; the internal pass must
use its already admitted InputBundle.

**Known provenance gap:** `NativeInput::derivations` currently returns empty for
relations without an embedded `derivation_id`. P3 primitive output still needs
source evidence for those relations. Settle this in the native construction carrier;
do not invent detached proof or replay the producer. Literal unit dependencies
must follow the actual selected physical definitions, not all-input fanout.

## P4–P10 boundary — HP09/HP10

- **P4–P6 incomplete:** the checked `PassOutput` change is not integrated. Existing
  callbacks/Cell projections remain. Complete P4 predicate contexts, generic P5
  port traversal/state binding and P6 requirement shapes/dependency closure.
- `predicates/coordinates::IndexEvaluator` still has its old synchronous/raw-map
  constructor. P7 already expects async
  `new(session, checked_inputs, registry, reserver, cancel, physical)`; implement
  that contract through native selection and explicit typed algorithm inputs.
- `p5/native_ports.rs` and `native_ports/{guards,walk}.rs` are **unlinked WIP**,
  not compiled. They need API corrections, terminal target/member projections,
  provenance and tests before linking. `inferred.port_binding_walk` is generated;
  a declaration is not evidence that the consumer works.
- **P7 incomplete:** `Inputs` and `RealizationOutput.rows` now contain checked
  batches; realization methods are async and take the actual session/physical
  inventory. `Inventory::load` uses native ordered keyed projection to typed rows.
  `p7.rs`, `seed.rs`, `evidence.rs`, `methods.rs` and `transfers.rs` still contain
  coupled raw/Cell graph/output work. Replace the seed, node remap, output and
  provenance path together using `native_rows`, `native_outputs` and declared
  node rewrites. Remaining sink calls use removed `into_rows`/old constructors.
- **P8 incomplete:** native `contexts` are linked with shared source binding;
  main uses NativeInput and actual derivations. `graph.rs`, `expansion.rs` and
  `expansion/**` still need the checked/native graph path and exact participation
  evidence. Main still assembles obsolete vector ports. The context code compiles
  in the last library check; full conservation/connection behavior is untested.
- **P9 incomplete:** `selections.rs` plus `selections/{native,bindings}.rs` uses
  actual native P6 winner/scope/state/method/provision joins and exact selection
  keys. Main/configuration still use old synchronous calls and
  `ConstructedInput`/`WorkspaceConstructor`. Complete checked async callers,
  parameter/descriptor routing, natural-unit conversions, kernel binding and
  source coupling; replace obsolete callbacks rather than adapting them.
- **P10 source replacement written, untested:** `p10.rs`, `production.rs` and
  `contracts.rs` now consume the real checked stage ports, shared physical inventory
  and RelationSymbolSource. Native joins select roots, indices and contributions;
  typed graph algorithms canonicalize under actual environments. Checked sinks
  and native remaps emit compiled roots, symbols, masks and coefficients.
  Dependency-cycle and derivative quantity predicates remain explicit algorithm
  preconditions. Auxiliary implicit-system/DAE relations use their own typed path.
  No P10 source error was reported in the last check, but integration tests,
  full physical meaning and complete derivations remain unverified.
- Removed P10 `Context`, `TypedRows`, `canonicalize_rows`, `methods.rs` and
  `reindexing.rs` are not compatibility APIs to restore. Old unlinked P10 fixture
  files and their xtask/test consumers still need deletion/replacement with real
  source-to-P10 workflows. P10 occurrence typing can map one old node differently
  under different consumer environments; preserve those actual root/consumer keys.

## Required foundation work after source integration

These issues remain in scope even after the compiler turns green.

1. **HP02/HP03 transfer:** `CompletedComputation::checked_relation` now caches its
   checked materialization and shares the actual buffers on subsequent access.
   Its first materialization still performs local field-value admission, and
   `NativeInput::validate` still compares workspace arrays. Finish property transfer from actual
   operations and owners, with only unresolved residual predicates. Do not expose
   an unchecked public constructor or replace value checks with hashes.
2. **HP04 ownership/accounting:** `FieldCheckedBatch::retained` now clones an
   already leased batch without another result claim. Audit native query export,
   which still attaches a conservative result claim alongside existing owners,
   plus algorithm-workspace forecasts and old unreserved concat callers. Preserve reservation-before-allocation and final-reader
   ownership without charging the same buffer repeatedly. Complete cancellation
   and native physical behavior tests; do not fix avoidable costs by raising caps.
3. **HP06 exact durable context:** in-memory memo dependencies now retain actual
   registry/input/document/policy/session owners and exclude volatile/stable
   computations from reusable results. Durable `memo/context.rs` still copies
   declarations, policy Cells/strings and document strings per stage; old
   16 MiB serialized/64 MiB retained limits remain. Replace with shared owners in
   memory and an explicit current invocation format on disk. Opening must restore
   actual selected policies, parameters, session semantics and auxiliary input
   bindings. The external producer path in `validator/producer.rs` still supplies
   `PolicySet::default()`; faithfully restored selected policies remain required.
4. **HP06 publication effects:** immutable operations have wakeable cancellation;
   mutable CAS publication still needs explicit visibility/durability outcomes.
   Local rename can become visible before a directory sync error; a remote write
   can lose acknowledgment. Retain intended bytes and reconcile actual state or
   report a truthful uncertain outcome. Never drop an in-flight mutable operation
   and infer that publication did not occur. Add fault-store tests for these cases.
5. **HP05/HP06 producer discipline:** private registered prepared/completed stage
   producers and local source completion are written. Test foreign context,
   incomplete ports, source ownership, failure and exactly one successful local
   producer invocation. Opening/import uses the real producer read-only before
   exposing a handle; inspection must not repair, publish or implicitly compile.
   ChangeSet persistence now uses complete staged batches and exact operation-row
   ordinals in wire version 2. Actual full rename and reopen have passed. Continue
   deleting any remaining verification staging/callback remnants after replacing
   their actual consumers.
6. **HP03/HP06 observations and residuals:** actual native logical/physical
   observations exist, but PassRecord persistence still stores logical explain
   data, P0/P1 observations are incomplete, and the P2 commit path still uses the
   preparation observation. Carry actual completed observations and preserve
   terminal success if later publication fails. `validate_affected` now selects
   obligations by changed inputs (including negative scopes); add referenced-table
   FK/negative-scope tests. Driver preconditions still repeat full P2 validation;
   select only declared unresolved obligations instead.
7. **HP07 closure:** focused native rule/delta/support tests pass. Complete real
   compiler use, independent supports, false/unknown/conflict outputs, negative
   scope invalidation and deletion/support-loss recomputation of complete affected
   strata. Delete obsolete row state and constructed callback remnants after their
   callers are replaced. An addition-only closure is not retraction support.

## Remaining product and closure work

- **HP08:** physical packages and shared inventory are source-written; old duplicate
  physical decoders are deleted. Run the changed quantity/MathIR adapter tests and
  primary-source NIST JANAF cp/h/s checks at 300/400/500 K. Establish actual states,
  units, affine/reference conversions, material definitions, methods, preconditions
  and domain failures; metadata agreement alone is insufficient.
- **HP11:** EngineSettings, immutable reader/owner/error/cancellation bindings,
  generated introspection stubs and Python stream tests are source-written. Run
  `just py-sync`, `just doctor`, `just python-stubs --check`, targeted
  `just py-test` and `just quality` only against the rebuilt extension. Verify the
  exact recipe syntax from `just --list`. New xtask stub tests are still unrun.
- **HP12:** no real complete FTPx/FcTP heater+mixer P0–P10 workflow, persistence,
  reopen, Rust/Python inspection or target performance receipt exists. Build fresh
  expected indexed graph/provenance cases from independent declarations. Exercise
  actual P10 rather than standalone graph fixtures. Measure actual target work,
  allocation/parse/producer counts, cold/warm behavior and configurable resources.
- **HP13:** delete old P10 and validator fixture routes, callback/row bridges,
  obsolete tests, golden stores and active narrow-profile assumptions. Regenerate
  target artifacts. Run Plan 05's terminal recipes, including `just ci-pr`,
  `just test-release` and `just parity-container`, then the focused actual-code
  G1–G7 review. These were not run to completion on this source tree. Full
  wheel/sdist qualification remains manual/release-time, not a new checkpoint gate.

## Verification receipts

Baseline is **0 failures** for every command. Rust test commands below use the
nextest **default** profile and explicit Arrow `force-validate` supplied by
`test-package`. Filtered tests are not evidence about unexecuted cases. Logs under
`build/` are local regenerable artifacts, not durable source authority. Subsequent
source changes mean earlier green receipts are historical focused evidence only.

| Evidence | Command / conditions | Result |
|---|---|---|
| **Tested** — rules | `just test-package pse-rules --test invariant_execution --test relational_execution --test native_delta_execution --test ordered_aggregate_unnest --test native_construction --no-fail-fast --status-level fail --final-status-level fail` | 27 passed, 0 failed/skipped; run `d3585816-d710-43de-9042-90b83141c320`; `build/hard-pivot-rules.log`. |
| **Tested** — authoring | `just test-package pse-authoring --lib --test change_set --test p0 --test targets --test load_package --test owned_documents --test rename --no-fail-fast --status-level fail --final-status-level fail` | 31 passed, 0 failed/skipped; run `e882e3af-3ae9-44b7-b0db-78292e75ded2`; `build/hard-pivot-authoring.log`. This supersedes the earlier 30/31 result; source-key getter was added later. |
| **Tested** — catalog fields/ordinals | `just test-package pse-catalog --test indexed_arguments --test filter_refinement --test native_output --test native_nested_output --no-fail-fast --status-level fail --final-status-level fail` | 6 passed, 0 failed/skipped; run `ab7b295b-37bc-4010-896b-523d291b941d`; `build/hard-pivot-native-fields.log`. Indexed reorder test uses one thread/four partitions. |
| **Tested** — scalar construction | `just test-package pse-catalog --lib scalar::index_tuple scalar::list_concat --no-fail-fast --status-level fail --final-status-level fail` | Final rerun: 4 passed, 0 failed, 102 filtered; run `1d753a83-e538-4a77-ab62-a8de554f4300`; `build/hard-pivot-restart-scalars.log`. Two concat failures in the first attempt were fixed before this rerun. |
| **Tested** — generated columns/ownership | `just test-package pse-relations --test generated_contracts --test typed_collection --no-fail-fast --status-level fail --final-status-level fail` | 13 passed, 0 failed/skipped; run `5980567d-b8c4-4870-9f37-a201e370c81e`; `build/hard-pivot-restart-relations.log`. Includes reserved concat/canonicalization and all three typed collection cases. |
| **Tested — incomplete generation** | `just codegen-bootstrap` | Rust-contract step succeeds; full step fails with 57 compiler diagnostics, exit 101. Latest narrow import correction is reflected in the subsequent check. `build/hard-pivot-restart-codegen.log`. |
| **Tested — failing build** | `just check-library pse-compiler` (dev, locked, library only) | 56 compiler errors, exit 101; no tests executed. `build/hard-pivot-restart-compiler.log`. |

**Tested — checkpoint documentation:** `just docs` built successfully with the
existing large-search-index warning. `just lint-typos`, scoped `git diff --check`
and a scoped `.venv/bin/python` check of six documents/136 local links/front matter
and whitespace found zero failures, baseline 0. All implementation agents stopped;
the final process inventory contained no native build/test/codegen or mdBook task.
No HP12/HP13 terminal acceptance or formal proof is claimed.

The 56 compiler diagnostics are distributed as follows: `p3.rs` 24; P4, P5 and P6
one each; `p7.rs` six, P7 seed five and transfers one; `p8.rs` one and expansion two;
`p9.rs` seven, configuration five and outputs two. This is a restart locator, not
a waiver or estimate of total remaining semantic work.
