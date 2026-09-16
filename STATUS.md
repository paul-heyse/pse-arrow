# Project status

## Current implementation direction — schema-first architecture pivot

Updated 2026-09-16 after the maintainer narrowed the requested work to completing
the architecture of the existing codebase, with no additional simulator functionality.

[Plan 08](docs/plans/08-schema-first-native-data-pivot.md) is the active
execution sequence: exact recursive field contracts, typed alternatives/quantities,
keys/references/provenance, coherent expression and numerical values, native checks
and durable declarations, then complete replacement of provider/compiler/store/Python
callers, native change/retention and architecture qualification. SP00/SP01/SP02 and the
local-check foundations of SP06/SP07 are active
under proposed ADR-0069 and blueprint revision 40. No package or terminal gate is
certified complete.

**Implemented:** shared exact recursive Arrow field admission replaces the duplicate
relation comparator and permissive Delta guard. Durable native field/schema descriptors
support checked cold reconstruction; incompatible extension annotations and lossy decode
are refused. Tests cover live Delta round trips, SQL mutations, nested field identity,
metadata scopes, masked parents and float precision. See the
[Plan 08 checkpoint](docs/plans/08-schema-first-native-data-pivot.md#implementation-checkpoint--2026-09-16)
for commands, counts, conditions and limitations.

**Implemented — registry cut:** native `FieldContract` replaces the deleted
`ColumnSpec`/`LogicalType` tuple model. Nested facets, metadata and nullability flow
through registry identity, Arrow binding, generated Rust/Python/JSON views and callers.
Canonical PSE enums now use string storage in execution and Delta; dictionary-specific
builders and caller assumptions are removed. Bound child enum meaning and Python IPC
storage are declared explicitly.

**Implemented — signed extensions and native local checks:** ordinal references and
source-span offsets use signed storage with generated range constraints. Publication
local checks and declared Delta member-table checks use the same native DataFusion
predicates. The monolithic Cell validation callbacks on these routes are deleted.
Member tables persist identity and SQL constraints before the first data write, and
their recorded declarations reopen without the registry. A narrow adapter executes
native collection expressions that the pinned Delta CHECK parser cannot parse; the
expressions and required dialect are persisted and cold-compiled. Scalar checks run
with only native Delta/DataFusion functions. Exact implementation, limitations and
verification are in the [latest Plan 08 receipt](docs/plans/08-schema-first-native-data-pivot.md#signed-extensions-and-native-value-predicates--2026-09-16).

**Tested, baseline 0:** the seven-package run passed 586 tests with force validation,
0 failures/skips. Python unit/component tests pass all 83 cases. Workspace compilation,
scoped Clippy, quality, formatting, regeneration and environment diagnostics pass.
All 9 targeted engine cases pass in CI mode with force validation; the final metadata-key
refinement also passes 202 catalog tests and the cold-source engine rerun. Exact
commands and conditions are in Plan 08. Ordinary unsigned PSE scalars, alternatives/
quantities/keys, coherent math/numerical rows, general invariant/domain transfer,
complete table/root policies and compiler/store/Python replacement remain open.
Full workspace Clippy and terminal architecture gates remain open.

The plan covers every finding in the schema engineering review and eight additional
opportunities grounded in current code and the pinned DataFusion/Delta skills. It
requires deletion of predecessor mechanisms with their caller cuts. Existing numerical
and Ipopt behavior is retained/restructured; new case/structural/initialization behavior,
source-to-solve simulator expansion, generated Pyomo and NL/SOL are excluded.

The Plan 07 checkpoint below remains evidence of implemented components and open gaps.
Its future simulator requirements are unscheduled scope, not Plan 08 acceptance gates.
SP00 has recorded the changed direction and decisions; fixture and terminal-command closure remain open.

## Plan 07 implementation checkpoint — before architecture-only re-scoping

Updated 2026-09-16 following the Plan 07 completion assessment.

**Implementation incomplete: 0 of 13 packages closed.**
[Plan 07](docs/plans/07-unified-datafusion-delta-hard-pivot.md) records the earlier
broader implementation. UD00–UD05 and UD07 are implemented in part; UD06 and UD08–UD12 remain open.
No M1–M5 milestone or G1–G7 gate is closed. ADR-0068 and blueprint revision 39 record
the authorized target.

The [completion review](docs/design_review/reviews/design_review_unified-datafusion-delta-completion_2026-09-16.md)
records **Revise**: G1 fails for the required single product lifecycle; G2–G7 remain
unresolved. It contains the package matrix, live deletion ledger, provider-level gaps,
all original F01–F14/V01–V13 dispositions and dependency-ordered functional cuts.

**Implemented in part:** combined exact Delta/kernel dependencies, composed native
planning, generated Arrow/Delta layouts, validating writes and SQL mutation hooks,
conditional exact-version publication/control reconciliation, source text in Delta,
owned fact providers, native finite inference, prepared numerical expressions/Jacobian
and an actual native Ipopt operator. The separate source-blob API/path is deleted.
The solver now handles real numerical runs, structured failure, callback panic,
cancellation and output ownership inside native execution.

**The main product path is still the predecessor architecture.** The compiler uses
custom snapshots, stage publication, contexts and artifact memo/hints. Python opens
the old local catalog/ref/manifest identities. Root `OperationNode` callbacks and the
closed relational rule/type/trace algebra remain. Their replacement/deletion is an
unmet UD01–UD05 obligation, not work to postpone until UD12.

**The native solver is not yet a source-derived simulator.** Tests install its planner;
the product runtime does not. UD06 case/structure/scaling/initialization is incomplete,
Pyomo/NL backends remain stubs, and native run outcomes are not yet published through
the complete target lifecycle. The numerical mixer/heater test uses constructed
expressions, not source-derived Slice A cases. Full kernel/derivative coverage remains
open. A foreign-memory allowance does not establish an actual C-allocation bound.

**Other required gaps:** common policy for every public execution route; preservation
of native configuration; complete model/case/run admission; member-write retry and
reconciliation across composed publications; exact member reuse/slices; semantic
CDF/dependencies/reuse; reader pins and data/log maintenance; cold OS-process
Rust/Python opening; target descriptors/reconstruction; full scientific and cost proof.

**Latest bounded execution receipts — Tested, baseline 0:**

| Command | Mode | Result |
|---|---|---|
| `just native-solver-test --no-fail-fast` | default, Ipopt + force-validate, digest-pinned solver container | 12 passed, 0 failed, 0 skipped |
| `just test-package pse-catalog --lib --no-fail-fast` | default / force-validate | 113 passed, 0 failed, 0 skipped |
| `just test-package pse-backend-native --no-fail-fast` | default / force-validate, Ipopt off; before final linked cancellation refinement | 2 passed, 0 failed, 0 skipped |

Plan 07 and review §9 record run IDs, durations, solver image and limits. These are
implementation-session receipts reviewed during the documentation audit, not new
audit-time test runs or terminal simulator acceptance. Earlier source-to-P10 tests
still exercise the old compiler/store and cannot close the target path.

**Current verification limits:** the environment diagnostic has 1 freshness failure
(baseline 0), and the editable extension is stale. Earlier compiler Clippy findings,
missing invariant fixtures and the transitive `proc-macro-error2 2.0.1`
future-compatibility warning have no final clearing receipt. Focused checks do not
establish current whole-workspace/Python health. The terminal recipe exists, but its
`unified_simulator` Rust and Python test targets remain absent.

**Next functional cut:** make exact Delta publications and common native preparation
the only source/model opening and compilation path, moving reusable transforms and
opening callers together and deleting the old store/stage/memo/manifest lifecycle.
Then complete native process/problem construction, actual solver/backend result
publication, target change/retention/inspection and full qualification. Deletions
belong to each cut; UD12 verifies their completion. There is no compatibility path,
historical-object migration or transition period.

This update is a documentation-only assessment. Earlier unrelated edits are preserved.
The material below is historical and does not certify Plan 07 or prescribe its execution.

## Historical Plan 06 implementation checkpoint

Recorded 2026-09-15 during the earlier provider-contract hard pivot.

**Implementation was in progress; full target acceptance remained open.**
[Plan 06](docs/plans/06-provider-contracts-hard-pivot.md) was that execution sequence.
PC00 authority changes and PC01–PC06 foundations are underway. No commit or push
has been made. Existing staged work is preserved.

**Latest implemented cut:** document/sidecar/artifact/change-set/stage-index operations
now use native preparation and execution. Publication inputs are immutable, nested
write failures retain the complete visibility journal, and private DML returns a new
validated immutable generation. Native CREATE TABLE defaults and existing prepared
readers survive that transition. Native EXPLAIN now uses those deferred hooks without executing mutation.

**Tested:** the Plan 06 receipt section records exact `test-package` commands and run
IDs for **18** storage tests, **9** publication/local-ref tests, **1** journal test and
**18** provider tests (default/force-validation/baseline 0, 0 failures). The provider
receipt now includes the EXPLAIN adapter (run `ecd37c9b-7ca4-4589-8119-41498d59fc2a`). `just clippy` remains a failed gate with
74 compiler-crate errors; the interrupted full source/P3 selection needs a new run.
No PC00–PC12 package or terminal engineering gate is claimed complete.

**Implemented:** blueprint revision 38 and proposed ADR-0067 describe the provider
framework. A shared binding index replaces duplicate source/role/table inventories;
native hierarchy registration, general provider binding, scoped policies, explicit
owned streams and prepared private DDL now use common session preparation.
Policy requirements lower through the existing invariant compiler; integration and
broader conformance checks are still underway.

**Implemented — current native cuts:** every pass has a native session and declared
effects; the optional `executes_plans` path is deleted. Multi-output producers run
once in the native plan, with independently readable completed output providers.
DDL destinations participate in policy before creation, including quoted scopes.
Cold traversal shares source/physical owners. Rust inspection consumes the common
owned native stream, and Python uses the same session/executor. Catalog constructors
now require their shared session factory; Driver no longer accepts a second factory.

**Implemented — store operations:** cold resolution, bundle/completed-production
publication and conditional ref updates now prepare native plans before their
backend work. Ref writes retain exact expected/intended checksums and typed actual
visibility/durability outcomes. All **8** focused publication/local-ref tests and
**17** cold-open/inspection/stage-port tests passed, default/force-validation,
baseline 0, with 0 failures/skips. Plan 06 records exact commands and run IDs.
The following cut removes the invariant validator's independent factory and passes
the actual operation session into every validation callback; verification is open.
The subsequent scope probe and ref tests passed **9/9**; updated cold/sidecar tests
passed **8/8**, and invariant execution passed **7/7**. Plan 06 records commands,
conditions and run IDs; these are bounded default/force-validation/baseline-0 results.

**Implemented — general provider capture:** native providers can be explicitly
captured before their key claims reach optimization. Actual complete rows establish
primary/unique constraints and remain immutable after original-source mutation.
The **15/15** provider tests pass, including nullable unique keys and unadmitted
view rejection. Broader facts, nested value layouts, resolution/metadata, private
DML, source orchestration and final engineering/quality/review gates remain open.

**Tested — Python bounded receipt:** after `just py-sync` and `just doctor`,
`just py-test python/pse/tests/test_snapshot_streams.py -q` passed **18** tests with
0 failures, unit/component/xdist-auto/baseline 0. This predates native store and
validator changes; current Python acceptance requires a new editable build.

**Tested — latest bounded behavior:** 18 stage/terminal/memo tests passed (run
`7d1f967a-280e-45dd-bcf7-af41bcfa800f`); 7 invariant tests passed (run
`d1bf6a28-cc8c-4c75-a3c8-cca6dc16e562`); 17 inspection/stage/normalization tests
passed (run `86f385b1-711d-4682-9c59-24f938a7895c`). All used the `test-package`
recipe, default/force-validation/baseline 0, with 0 failures/skips. Exact commands,
conditions and subsequent-change limits are in Plan 06's implementation progress.
`just check` after the catalog constructor cut: workspace/all-targets/dev/locked,
0 errors/warnings, baseline 0. The editable extension requires another refresh; Python
behavior and the complete engineering/terminal gates remain open.

**Tested:** `just test-package pse-catalog --test provider_contracts --test
native_input_ports --no-fail-fast --status-level fail --final-status-level fail`,
default/force-validation/baseline 0, **9 passed, 0 failed/skipped**, run
`5f5bb9e0-8156-4f7d-8514-e0e18a9fb64e`. This covers provider/SQL admission, view
binding, unproved-key refusal, root registration, private prepared DDL, policy
composition/settings and stream ownership. Subsequent namespace declaration edits
and the rule integration await their next checks; this is not PC12 acceptance.
`just codegen-bootstrap` regenerated all three schema targets. `just adr-lint`:
67 ADR records and 31 register rows, 0 findings. Terminal suites remain open.

## Historical provider-pivot boundary

The [provider review](docs/design_review/reviews/design_review_provider-contracts_2026-09-15.md)
requires **Revise**, with G7 failed and G1–G6 unresolved for the expanded target.
Plan 06 covers all nine findings, ten review packets and every provider level in
13 ordered implementation packages, with explicit deletion and acceptance checks.
One Codex stream builds the target; no parallel legacy architecture is retained.

**Tested — review characterization:** five probes passed with 0 failures/skips and
0 compile warnings, default/force-validation/baseline 0. They include assertions of
existing defects and do not establish target conformance. The exact command and
conditions are in the [probe receipt](docs/design_review/evidence/provider-contracts-probes-2026-09-15.md).

**Implemented and Tested — latest local correction:** the engineering-inspection
attempt failed at P3 because P0's internal `packages` role collided with a real
input port. P0 now uses `package_dependency_headers`. Its focused regression:
`just test-package pse-authoring --test p0 --no-fail-fast --status-level fail
--final-status-level fail`, default/force-validation/baseline 0, **2 passed,
0 failed/skipped**, run `f91e8775-c782-4f6b-aa94-66a94bffb710`.
The full engineering workflow and Python inspection were not rerun after that fix.
The editable extension has not been refreshed for that correction. This local
rename does not implement the scoped provider binding design.

**Environment — 2026-09-15:** the exact `uv` requirement and every exact pin of a tool
that only executes (quality/test groups, cargo tools, CI tool installs) were removed;
tools carry floors and run at what `uv.lock` resolved, and CI installs current releases.
`just doctor` no longer asserts tool releases. Plan 06 PC10 still refreshes the target
editable extension. Earlier environment/extension success below is historical.

## Historical Plan 05 implementation

The following records prior implementation states and receipts. Later changes and
the current boundary above take precedence; these are not fresh acceptance claims.

**Implemented, incomplete:** actual source commits run the native P3–P10 pipeline.
P3 uses admitted inputs, owned source documents and typed child configuration values.
P4/P5 consume P3's admitted selected-child correspondence. P6 uses native finite
requirement, scope, dependency and coordinate plans. P7–P10 consume current graph
inputs for their target mathematical construction and emit typed, source-coupled
outputs. The four heater/mixer source-to-P10 graph expectation tests pass. Broader
engineering, provenance and cold inspection acceptance remains open.

**Implemented and Tested:** reusable logical-plan UDFs preserve established field
meaning across scalar/nested selection and native aggregation. UNION keeps the actual
intersection of input annotations and permits native primitive type coercion.
Registered stage producers execute private completions; admission uses declared input
ports plus primitive model/case scope. Repeated output ports retain their own local
keys and require explicit cross-port references.

**Implemented and Tested:** shared immutable native rule computations retain actual
owners and grouped source witnesses. Absence scopes deduplicate by exact relation and
port. `RulePlan::Assert` explicitly classifies truth; ordinary filters select rows.
The implicit root-filter truth shortcut and duplicate truth lowering are deleted.

**Implemented and Tested:** P7 consumes the shared physical context, including
Boolean point types and its admitted neutral type. Fixed/indexed subject alternatives
use one generated rule; P7 checks actual axis domain kinds. Durable context values
are captured once per request engine mode and shared with hints under one owner.
Actual producer, attempt-record, reopening and detached-context tests pass.

**Implemented — deletion:** predecessor injection, the old P10 graph importer,
replay constructors, duplicate graph/row adapters, golden-store generator and value
observations are deleted. All remaining stores under `tests/golden` are deleted.
Python inspection fixtures now create fresh current stores through the actual Driver.
Retain graph inputs only when the target computation uses them; no legacy runtime
path or predecessor-equivalence acceptance campaign is intended.

**Implemented — native schema evolution:** the row interpreter, automatic
version-path search and generated migration lookup are deleted. Explicitly selected
registered edits now become native projections and reusable nullability UDFs.
Nested defaults retain their actual semantic child fields; source roles bind exact
declared versions. Residual relation obligations remain before publication.

**Implemented and Tested — deletion:** the broad `TargetContext` object
and its nine-family decoder are deleted. Lexical binding retains only actual entity,
symbol and template-domain inputs alongside its single instance inventory. Rename
loads only its entity ancestry. P3 lowers from borrowed generated Arrow columns,
without constructing a second complete target inventory. Tests build actual checked
Arrow relations directly.

**Implemented — durable invocation bindings:** the manifest now selects one immutable
checksum-addressed binding containing exact parent encodings, the selected original-document source, policy owners
and sealed engine settings. Publication, explicit-context admission, pinned reopening
and durable reuse consume that binding. The target-keyed receipt and default-policy
replay fallback are deleted. Native engine restoration retains the supplied actual
implementations, restores explicit absent settings and refuses incompatible context.
Invocation/configuration allocations retain their own reservations. The copied stage
context has since been deleted; wider HP06 lifecycle/implementation-binding gates remain open.

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

## Historical Plan 05 verification

Failure baseline is **0**. Rust tests use Arrow **force-validation**. These focused
receipts establish only their named scope; exact conditions and earlier results are
in [Plan 05's current integration section](docs/plans/05-native-logical-plan-hard-pivot.md#current-source-and-port-integration--2026-09-15).

| Evidence | Command, mode and result |
|---|---|
| **Tested — source template and heater boundary** | `just test-package pse-tests-engine --test native_template_graph --test native_engineering_workflows --profile ci -E "'test(two_templates_produce_declared_symbols_expressions_and_equations) \| test(heater_ftpx_source_to_canonical_graph)'" --no-fail-fast --status-level fail --final-status-level fail`: **1 passed, 1 failed, 3 filtered**, 231.447 s; run `28301a56-79fb-4e13-a166-80d0f5238e1f`. Two-template source-to-P10 passed. Subsequent heater failures in P4/P6 were corrected. |
| **Tested — engineering source-to-P10** | `just test-package pse-tests-engine --test native_engineering_workflows --profile ci --no-fail-fast --status-level fail --final-status-level fail`: **4 passed, 0 failed/skipped**, 599.359 s, ci/force-validation; run `becb477b-1262-4c5c-82cc-81bd527c7d45`. Heater/mixer FTPx and FcTP configurations pass the independent graph expectations after explicit state/law/pressure broadcasts and the indexed absolute-pressure difference declaration. Predates subsequent helper cleanup; complete cold reopening, Python and numerical acceptance remain open. |
| **Tested — quantity and indexed bounds** | `just test-package pse-compiler -p pse-quantity --no-fail-fast --status-level fail --final-status-level fail`: **105 passed, 0 failed/skipped**, default/force-validation, 2.781 s; run `3b592ad9-ea8f-431f-a625-83109e05d376`. Includes scalar/single-axis/multiple-axis bound inference and refusal of missing Broadcast. Predates the latest realization refactor and generator changes. |
| **Tested — compiler, rules and authoring** | `just test-package pse-compiler -p pse-authoring -p pse-rules --no-fail-fast --status-level fail --final-status-level fail`: **137 passed, 0 failed/skipped**, default/force-validation, 11.417 s; run `6480e468-1e0b-432f-b50f-66efa2cacaf6`. Includes the `TargetContext` deletion, physical datum/basis composition and native helper refactors. The newer compiler/quantity receipt covers indexed-bound inference. |
| **Tested — foundation integration** | `just test-package pse-compiler -p pse-schema -p pse-rules -p pse-mathir -p pse-quantity -p pse-relations -p pse-catalog -p pse-authoring -p pse-templates --no-fail-fast --status-level fail --final-status-level fail`: **567 passed, 0 failed/skipped**, default/force-validation, 14.926 s; run `c5ff2ea8-f3f4-4a2b-a700-6ede82f27df3`. Includes the shared realization entry point, indexed-pressure difference, P3/P10/native-construction helper cleanup, borrowed operands and generated adapters. Includes the admission-binding pivot and producer helpers. |
| **Tested — current admission bindings** | `just test-package pse-tests-engine --test catalog_pinned_reopen --test catalog_session_codec --test catalog_store_protocol --test stage_ports --test native_normalization --profile ci --no-fail-fast --status-level fail --final-status-level fail`: **31 passed, 0 failed/skipped**, 244.627 s, ci/force-validation/baseline 0; run `9bc1efe9-9ae0-408d-adee-77e1eedd145e`. Includes selected-policy P3 cold reopen, explicit-context refusal, engine setting restoration, exact parent encodings and owned lifetimes. The later 3-test `stage_ports` run also passes, including restart reuse, fresh attempt records and release after the memory store and its input owners are dropped. |
| **Tested — catalog integration** | `just test-package pse-tests-engine --test catalog_inspection --test catalog_pinned_reopen --test catalog_session_codec --test catalog_store_protocol --no-fail-fast --status-level fail --final-status-level fail`: **26 passed, 0 failed/skipped**, default, 15.376 s; run `1d3b9010-4d2a-4619-8160-e69cd871c51b`. Native integrity, registered producers, current receipts, owned streams and diagnostic codec bindings. |
| **Tested — rule execution** | `just test-package pse-rules --no-fail-fast --status-level fail --final-status-level fail`: **57 passed, 0 failed/skipped**, default, 8.205 s; run `7c2e535b-3817-441e-a21d-5e3e42dc5c7f`. Includes explicit truth, ordinary selection, exact payloads, source witnesses and evolving deltas. |
| **Tested — schema** | `just test-package pse-schema -p pse-relations -E "'package(pse-schema)'" --no-fail-fast --status-level fail --final-status-level fail`: **85 passed, 0 failed/skipped**, default, 4.470 s; run `2cf96206-09de-441d-a1e5-51781c27193e`. Includes explicit assertion scope and current indexed-product contracts. |
| **Tested — governance boundary** | `just governance`: **59 passed, 0 failed/skipped** before its generation-hygiene step failed on untracked generated `property_read_occurrences.rs`. Shared Git index preserved. Full governance remains incomplete. |
| **Tested — compiler library** | `just test-package pse-compiler --lib --no-fail-fast --status-level fail --final-status-level fail`: **25 passed, 0 failed/skipped**, default, 2.181 s; run `723bb83b-c57e-4560-95bb-fd4f5ac8c3e8`. Includes Gather lexical binding and exact typed parent-to-child transfer. |
| **Tested — registered ports and contexts** | `just test-package pse-tests-engine --test stage_ports --no-fail-fast --status-level fail --final-status-level fail`: **3 passed, 0 failed/skipped**, default, 10.882 s; run `6cd56345-f4f4-4cd7-b80b-e1d07a014896`. Actual invariant validation, publication, Driver records, reopening and shared ownership. |
| **Tested — enum admission** | `just test-package pse-relations --no-fail-fast --status-level fail --final-status-level fail`: **29 passed, 0 failed/skipped**, default, 1.298 s; run `6d46f9de-6f3e-4def-ac28-1aba5d119f1b`. Dictionary membership and recursive field-value admission remain checked. |
| **Tested — subject invariants** | `just test-package pse-rules --test invariant_execution --no-fail-fast --status-level fail --final-status-level fail`: **6 passed, 0 failed/skipped**, default, 6.458 s; run `b3ecdc9d-0405-46eb-9ec2-6e21a720ba88`. Fixed/indexed coordinate alternatives and exact invalid keys. |
| **Tested — fields and native outputs** | `just test-package pse-catalog --test native_union_metadata --test native_selection_output --test native_nested_output --test native_output --no-fail-fast --status-level fail --final-status-level fail`: **13 passed, 0 failed/skipped**, default, 3.572 s; run `dfca4dda-c1b7-4929-9c00-40991fb99ef7`. |
| **Tested — static checks** | Current `just quality`: **0 findings/errors**, 14 setup-guard tests passed. Earlier `just check`: 0 errors and one PyO3 deprecation; explicit opt-out corrected. Generated, catalog and template production/test lint causes are corrected. The latest `just clippy` run clears rules and authoring and reaches 71 compiler library findings (72 with library tests). This includes admission bindings but predates the document-source follow-up; workspace Clippy remains incomplete. Baseline 0. |
| **Tested — Python current store** | `just py-sync` rebuilt the editable dev extension and regenerated stubs against the actual API. `just py-test -n 0`: **80 passed, 0 failed, 55 parity tests deselected**, 48.83 s, Python 3.14.7, unit/component mode, baseline 0; its recipe first publishes and admits two fresh current snapshots. `just check`: all workspace targets compile, 0 warnings/errors. `just doctor`: environment ready, matching lockfiles. |

## Remaining work

Execute Plan 06 PC00–PC12. It carries all unfinished HP00–HP13 requirements,
including:

- Extend the passing FTPx/FcTP heater/mixer graph workflows to complete physical
  expectations, provenance, cold reopening and Rust/Python inspection.
- Complete durable parameter, auxiliary implementation and execution-effect dependencies;
  finish publication/cancellation semantics and shared comparison-work accounting.
- Semantic property transfer, allocation ownership and measured work accounting;
  complete publication/cancellation/durability outcomes.
- Fresh generation checks, editable-extension refresh and Python acceptance,
  workspace checks, release/parity, target measurements and final G1–G7 review.
- Final legacy code/data/caller audit and terminal closure. No full-plan acceptance
  or formal proof is claimed.

Family checks previously passed; governance remains incomplete at generated-file
tracking hygiene. Full parity, Rust workspace lint, measurements and terminal
architectural acceptance remain open. Use the current environment boundary above.

## Working tree

Preserve branch `wave2/semantic-compilation`, initial HEAD
`eebc82e9f2298e4a53e08818cfd2a1dfb6fc310b`, and its substantial staged, unstaged and
untracked work. Reuse current interfaces only when they fit the target. The next
implementation follows Plan 06; this task completed documentation only.
