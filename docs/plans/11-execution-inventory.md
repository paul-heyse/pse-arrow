---
title: Integrated native performance execution inventory
status: in-progress
date: 2026-09-19
adrs: [ADR-0074]
phase: 1
evidence: Implemented I00-I17; Tested 332 targeted units after I18 corrections; final qualification open
---

# Plan 11 execution inventory

**Historical inventory:** [Plan 13](13-rust-computation-architecture.md) owns current
execution and carries every unresolved acceptance obligation. See the
[W19 repair checkpoint](13-w19-repair-checkpoint.md) for resumption. Original package
statuses and failed/incomplete receipts below are preserved at their source boundaries.

[Plan 11](11-integrated-native-performance.md) is the scope authority. I00-I17 are implemented for the
current authorized scope. Targeted units suffice for all implementation and
deletions; integration remains not run by design until the complete I17 barrier.
Starting I00-I03 source capture: `build/plan11/start-20260920T002530Z/`.
I04-I06 source capture: `build/plan11/i04-i06-start-20260920T015405Z/`. No historical receipt
qualifies changed source. Baseline is zero failures.

The previous [I07-I12 execution checkpoint](11-i07-i12-execution.md) records the
implementation, deliberate capability boundaries and 70 passing isolated units.
Its starting capture is `build/plan11/i07-i12-start-20260920T034119Z/`; final unit
commands and logs are in `build/plan11/i07-i12-final/`.

The [I13-I15 execution checkpoint](11-i13-i15-execution.md) records exact
native-write settlement, completed model retention and bounded scheduling, and the
Python startup/stream boundary. **57 Rust force-validation units and 65 Python units
pass**, with normal workspace/static checks and pure generation green at baseline
zero. Receipts are in `build/plan11/i13-i15-final/`.

The current [implementation improvement checkpoint](11-implementation-improvements.md)
records the approved H01-H12 follow-up across I00-I15: typed retained selections,
selective obligations, early terminal demand, grouped compiler/rule execution,
numerical invalidation, native resident pruning, bounded scheduling and schema transfer.
**97 distinct Rust force-validation units and 32 Python units pass**, at baseline zero,
with normal workspace/static and pure-generation checks. Exact commands and receipts
are in that checkpoint and `build/plan11/improvements/`. Earlier receipts are historical
where these changes supersede their source. The [I16-I19 execution checkpoint](11-i16-i19-execution.md) closes I16/I17 with
74 registered unit groups and complete caller/deletion reconciliation. Its I18
correction checkpoint now records 332 passing isolated units. Continue with **I18**;
functional and performance qualification remain open.

## Packages

| Package | Deliverable | Status |
|---|---|---|
| I00 | Scope ledger, policy/decision amendments and command ownership | complete |
| I01 | Binding, witness and retained-extent correctness foundation | complete |
| I02 | Registry-owned schema/validation evidence and preservation | complete |
| I03 | Retained assembly/context and immutable selected views | complete |
| I04 | Distinct plan facts and relevant preparation witnesses | complete |
| I05 | Chunk ownership and typed completion foundation | complete |
| I06 | Native composition and finite checked output ports | complete |
| I07 | Demand, shared obligations and terminal contracts | complete |
| I08 | Bulk compiler construction and retained inventories | complete |
| I09 | Affected-key rule updates and epoch state | complete |
| I10 | Semantic indexes and canonical work reuse | complete |
| I11 | Guarded scalar program and solve workspace | complete |
| I12 | Exact Delta snapshots and qualified native cache families | complete |
| I13 | Evidence-aware native writes and publication settlement | complete |
| I14 | Model result retention and bounded scheduling integration | complete |
| I15 | Python startup, stream and metadata boundary | complete |
| I16 | Typed diagnostics, fixtures, observations and assessment tooling | complete |
| I17 | Full deletion, caller, policy and source barrier | complete |
| I18 | Complete functional assessment and grouped remediation | open |
| I19 | Performance characterization and final acceptance/outcome | open |

## Deletions

| ID | Replacement/deletion | Owner | Status |
|---|---|---|---|
| L01 | Invalid field-before-binding and broad witness alias scope | I01/I04 | complete |
| L02 | Duplicate schema serialization, cache-key derivation and successful-row path formatting | I02 | complete |
| L03 | Repeated local admission at certified handoffs | I02/I06/I07/I13 | complete |
| L04 | Nested factory/registry rebuild and witness-through-bound-state comparison | I03 | complete |
| L05 | Ancestor-path intrinsic fact rediscovery and independent duplicate root walks | I04 | complete |
| L06 | Compiler generic tuple layout/pack/member/UNNEST route and its fixtures | I06 | complete |
| L07 | Unconditional relational stage capture and all-intermediate output execution | I06/I07 | complete |
| L08 | Repeated invariant execution, existential full sort/DISTINCT/collect, settled barriers | I07 | complete |
| L09 | Per-row queries, repeated generated-prefix union/concat, inventory/path scans | I08 | complete |
| L10 | Full rule representative recomputation for unchanged keys | I09 | complete |
| L11 | Repeated quantity/binder scans and unrequested combined canonical preimage | I10 | complete |
| L12 | Scalar callbacks' one-row stage/batch evaluator and captured solve token | I11 | complete |
| L13 | Global pointer ledger, repeated allocation-map sweeps and cache-read buffer wrapping | I05 | complete |
| L14 | Latest-head coupling/probes for exact snapshot/resident hits | I12 | complete |
| L15 | Success-path publication reconciliation scans and redundant write checks | I13 | complete |
| L16 | Blanket serial independent work and per-task full-machine budgets | I14 | complete |
| L17 | Python duplicate registry/import lint/schema field-removal loop | I15 | complete |
| L18 | Forced Diagnostic value helpers, display-code assertions, aborted case loops, masked tools | I16 | complete |

## Coverage and evidence

The [case manifest](11-acceptance-cases.toml) owns executable identities, modes,
positive/negative oracles and namespaced Plan 10 carry-forward references. Plan 11
T01-T14 and V01-V16 remain open until their owning packages supply current receipts.
Implemented case identities are in the manifest; full T01-T14/V01-V16 qualification
remains open. All historical failures, timeouts, interruptions and unrun gates listed by Plan 11
remain obligations; no ready environment or unit result rewrites those outcomes.

## Implemented boundary

### I00 — Decisions and execution scope

Implemented: ADR-0074, authorized blueprint revision 42, proposed-ADR/register scope
amendments, active-plan pointers, inventory and namespaced Plan 10 carry-forward
manifest. ADR-0074 remains **proposed**; the integrated review retains its Revise
verdict. No acceptance status, historical receipt or accepted ADR was rewritten.
The previously added Plan 11 campaign guard remains. No further campaign enforcement
or agent infrastructure is required. This is a sole-agent implementation checkpoint.

The pure generator's xtask dependency on `pse-engine` is now optional under
`package-fixtures`: contract generation can repair generated code without first
compiling its consumers. Dependency pins and the single type universe are unchanged.

### I01 — Binding, lexical scopes and retained allocation extent

Implemented: native lambda binding and coercion precede field reconstruction while
the offered schema remains the comparison authority. Original UNNEST depth/dependency
mapping is validated before native expression rewriting can rebuild it. Native
DataFusion context errors retain their typed causes. Compiler witness products project
their exact lexical scope before joining repeated short role aliases.

Known leased buffer wrappers resolve to the actual storage owner and full retained
extent. The 2,624-byte backing/408-byte visible-view regression is covered through
slicing and escape lifetimes. Foreign views cannot establish hidden allocation extent;
the external-admission path uses an explicitly reserved copy. Arrow's custom buffer
capacity is not treated as an ownership proof. Existing pushdown mitigation remains.
L01's binding/alias portion landed in I01; I04 now supplies relevant dependency witnesses.

### I02 — Registry-owned local evidence

Implemented: registry arenas retain exact Arrow `SchemaRef`s and relation contracts;
same-owner prepared validation uses direct schema/field maps. Nested traversal and
predicate schemas are retained. Successful rows retain compact path coordinates;
only findings format paths. Immutable reflection batches are admitted once per owner.
Generated accessors/builders use retained schemas through pure regeneration.

Checked clone, slice, reserved filter, exact projection and chunk concatenation preserve
the local proof they actually support. Projection runs missing target row predicates;
concatenation does not establish cross-chunk keys or completeness. Unchanged checked
handoffs have no additional local predicate evaluations in the targeted controls.
No scalar predicate interface was invented: the real prepared validator is batch based.
I11 still owns the numerical scalar program.

Admission-site audit (Implemented or retained by explicit treatment):

| Treatment | Current sites | Implementation boundary |
|---|---|---|
| Retained | Engine checked workspace/role inputs; `RelationSource::from_checked`; generated `View::from_checked`; registry reflection | Keep exact registry/contract and buffer owners; no repeated local value scan |
| Preserved | `FieldCheckedBatch` clone/slice/filter/exact projection/concatenation; generated checked views | Preserve only supported local evidence; row/key/reference obligations remain separate |
| Missing check | Computed/cast/joined fields; `CompletedComputation::checked_output`; compiler `admit_owned_projection`; checked scalar literal boundary | Establish the required local contract once for actual new values; completed outputs retain the checked result |
| Untrusted ingress | Factory raw candidate/workspace inputs; raw role batches; `RelationSource::from_family`; generated `try_from_batch`; external source batches | Validate exact schema/metadata/values and use the actual ownership admission contract |
| Replaced in I06 | Compiler tuple layout/member/findings/derivations transport | Checked ports carry declared chunk owners; newly constructed findings/derivations receive their own local admission |

L02's duplicate schema/preparation/path work is removed. L03 remains open across
I06/I07/I13; these later raw boundaries were not bypassed to claim early completion.

### I03 — Retained model context, selections and attempts

Implemented: a compatible factory/registry pair retains one `ModelAssembly` and its
actual `SessionContext`, native functions, planner/rules and configuration. Immutable
selection views retain actual bindings and effective policy. Namespace commands fork
their view; old selections retain their original providers. Nested `candidate_roles`
binds checked facts over shared capabilities instead of rebuilding a factory/registry.

`Witness::matches` compares retained owners directly and is infallible; it constructs
no context, catalog or policy inventory. Same-address mutable MemTable/observed sources
are refused as immutable witnesses. I04 now adds fine-grained consumed-input relevance;
I03 established retained owners without claiming whole-view reuse after any edit.

SQL planning clones the retained context. Actual execution binding applies any scoped
runtime, and each attempt gets fresh time, cancellation, cache and settlement state.
At DataFusion 55.1.0, the builder resets private prepared SQL registrations. Definition
commands therefore use the retained native definition state. Other command results
apply actual native registration/configuration changes to that retained state without
capturing attempt services or selected catalogs back into it. Native alias registration
is checked against the exact resulting function map. No lock crosses an await.

Delta-specific planner composition and `RequireSessionState` remain in catalog/runtime.
The engine gained no Delta dependency. I05 now supplies chunk ownership/ledger replacement;
I12/I14 own later storage/runtime composition and bounded scheduling.

### I04 — Native facts, preparation and relevant dependencies

Implemented: scoped traversal retains actual producer owners, derives expression
structure once per owner, and keeps lexical/outer/worktable/contract qualification
separate from ancestor paths. Grouped field admission and `prepare_many` share
related output roots and cache planning. Expression-field memoization compares the
complete expression, qualified schema and actual function owners; contextual
subqueries, outer references, variables and lambdas retain live use-site checks.
Changed native nodes are reconstructed and admitted; caller factories still have
input and actual returned-output admission.

Closed relational cache proofs retain consumed bindings, including empty/negative
reads, and selected function implementations. Unrelated binding/function revisions
can preserve them. Opaque operations, metadata resolutions and outstanding provider
requirements conservatively retain complete-selection proof. Effective policies,
planner/rule owners, schemas and registry identity remain relevant. Mutable or
observed providers cannot supply immutable evidence. This is reuse evidence, never
cached authorization or a skip of current allocation admission.

Pre-simplification stable/volatile dependencies survive native folding. Repeated
read/observe/nondeterministic preparations replay the retained original with fresh
query time; effectful preparations remain once-only. Round reset admits native
memory scans and specifically qualified families, rather than every DataSourceExec.
Dynamic filters remain disabled for reused round plans; Delta scan qualification
belongs to I12.

### I05 — Allocation owners and terminal completion

Implemented: `OwnedRecordBatch` carries an explicit producer-local `AllocationScope`.
Actual buffers retain safe per-allocation leases; the local provenance index contains
weak entries. Imports preserve the originating charge and reject conflicting live
owners. Unknown raw/foreign output uses a reserved copy; trusted native producers
transfer a pre-reserved allocation or explicitly admit native capacity. No process
global pointer ledger or ownership-discovery fallback remains.

Resident cache reads clone admitted chunks without rewrapping their arrays. Evicted
results keep only the allocations still referenced by exported arrays. Spill decoding
establishes its own allocation lease. Incremental unique-allocation totals replace
repeated cache-map sums. Explicit `compact` reserves for coexistence before an Arrow
`take`, retaining the compact result only when it reduces backing extent.

Completion distinguishes valid results, invalidity, cancellation, abandonment,
resource refusal, partial streams and uncertain effects while preserving typed
causes. Last-waiter abandonment remains terminal for that attempt; a fresh attempt
owns a fresh cell. A partial transport error does not overwrite a more specific
cancellation/resource/settlement classification. Delta settlement retains the known
commit version inside the typed cause; no durable mutation was executed for these units.

Allocation escape inventory (Implemented; all entries preserve actual buffer
lifetimes, with the following explicit admission boundaries):

| Escape | Owner/admission and source |
|---|---|
| Batch, column, projection, slice, nested validity/offsets/children and dictionary values | `pse-ids/src/owned_buffer.rs`: per-allocation BufferOwner and full backing extent; explicit compaction can release large parents |
| Checked relation clone/projection/filter/concat/canonical output | `pse-relations/src/columnar.rs`, `pse-ids/src/canon/api.rs`: carry OwnedRecordBatch; new kernels transfer admitted reservations; exact preserved field evidence remains separate from global keys |
| Native resident cache, cache eviction, spill reader | `pse-engine/src/session/cache.rs`: admitted chunks, incremental retained totals, decoded output leases; no buffer owns the entire cache bundle |
| Finite operation ports and composed query captures | `pse-engine/src/operation/ports.rs`, `session/input.rs`, `session/computed.rs`: typed chunk transfer or explicit raw-output admission; retained roles import allocation provenance |
| Query stream, raw external workspace/role/provider input | `session/preparation.rs`, `session/roles.rs`, provider binding: known scopes travel with bindings; unknown foreign storage copies under quota |
| Scalar/list/view adapters | `session/scalar/list_field.rs`, `list_concat.rs`: safe reservation attachment to actual escaping buffers; a later unproven raw handoff copies rather than inferring provenance |
| Numerical and solver Arrow output | `pse-numerics/src/expressions.rs`, `pse-backend-native/src/native/output.rs`: pre-reserved native allocations attach buffer leases before raw batch return |
| Arrow C stream and Python inspection | `pse-catalog/src/inspection/stream.rs`, `pse-py/src/inspection/stream.rs`: exported Arrow storage keeps leases after reader/handle close; isolated native FFI unit exercises last-array release |

### I06 — Visible relational segments and checked finite ports

Implemented: `Algorithm` composition hooks expose known-input native queries and
post-finite projections/joins. `ComputationPlan` captures a real physical child only
at its finite consumer; native qualified fields survive duplicate column names.
`RelationPlan` carries exact source support where unchanged and discards source
receipts on physical input replacement. Finite ports share the existing invocation
completion by actual producer owner, selected logical inputs, closed demand and
epoch. Atomic algorithms validate every declared output before any port is readable;
independent demand is explicit in the producer contract.

Sibling physical planning retains the same actual inputs. DataFusion cooperative
scheduling shells preserve identity only through their unchanged child or the exact
native cache completion owner. Genuine replacements create a fresh producer body
and completion. Transparent contract/cooperative shells can transfer checked chunks;
relational rewrites establish their new output fields normally.

The generic `native/layout.rs` tuple transport and all its compiler callers/tests
are deleted. Legitimate domain list/struct values and semantic UNNEST remain. Model
artifact selection now tracks actual produced relations. Known-input queries are
composed once into native children; direct nested invocations over newly generated
values use the same query builders after those values exist.

Body classification and current materialization boundaries:

| Segment / source | Relational composition visible before execution | Required finite or effect boundary |
|---|---|---|
| Source projection, document edit/rename (`native/source.rs`, `edit.rs`) | Native declared document/request children and output ports | Parser/AST binding, exact before-image edits, rename resolution and typed source projection require actual document values |
| P3 normalization (`p3/primitives.rs`) | Authored primitive projections and support-key fields | Syntax lowering, selected configuration, generated expression/seed construction and provenance depend on parsed/new values |
| P4 inference (`p4.rs`) | Declared native input graph; rule queries are built for actual finite-round bindings | Fixed-point computation remains finite; affected-key delta updates are I09 |
| P5 ports (`p5/ports.rs`) | Instance/template/path-length joins, missing-length obligation and declared port projection | Domain path/selector instantiation and subsequent generated-value construction |
| P6 indices (`p6/framing/indices.rs`) | Domain pool and index candidate joins, memberships and tuple selection | Index environment construction and changing generated/round inputs |
| P7 realization (`p7/inventory/links.rs`, `native_matches.rs`) | Native inventory correspondence join/union plans | Graph binding, kernel/symbol realization and source-support attribution; P9 augmented inputs bind the same builders when available |
| P8 law contexts (`p8/contexts.rs`) | Law framing, guards, choice and axis joins with explicit requirement children | Law expression lowering/contribution construction consumes the captured frame and actual source witnesses |
| P9 methods (`p9/selections/native.rs`, `bindings.rs`) | Requirement/resolution/scope/method/provision joins, uniqueness/orphan obligations and state-parameter selection | Selected-method expansion and dependent P3/P5/P6/P7 work require newly selected/generated values |
| P10 canonicalization (`p10/composition.rs`, `production.rs`) | Ordered finite inputs, contribution semijoin; output root-remapping joins and unchanged/auxiliary projections | Exact physical graph/type validation and MathIR canonicalization remain finite |
| Artifact publication (`pse-catalog/src/artifact.rs`) | Grouped named native output preparation | Requested completed-result retention and Delta effect settlement remain explicit boundaries |

I07 still owns full output-demand/obligation closure and elimination of unrequested
intermediates (the remaining L07 scope). I08 owns bulk construction and retained
inventory/index work inside finite bodies. Their work is not silently included in
I06 completion. No integration or performance outcome is inferred from these units.

## Verification

### I04-I06 checkpoint

Tested: **82 isolated Rust units passed, zero failures, baseline zero**, using the
normal `just unit-package` recipe with `pse-relations/force-validate`. The selected
catalog controls exercise typed errors and a local Arrow C stream, with no durable
Delta transaction. Compiler controls use tiny fake inputs or pure plan composition;
no process-model compiler journey ran.

| Command / exact selection | Passed | Log under `build/plan11/` |
|---|---|---|
| `just unit-package pse-engine 'package(pse-engine) and (test(session::reuse::) or test(session::freshness::) or test(session::traversal::) or test(operation::ports::) or test(operation::completion::) or test(session::admission::) or test(session::cache::tests::) or test(session::round::))'` | 41 | `i04-i06-engine-verified.log` |
| `just unit-package pse-engine 'package(pse-engine) and test(session::cache::admission::tests::)'` | 1 | `i04-factory-admission-verified.log` |
| `just unit-package pse-engine 'package(pse-engine) and (test(session::contract::tests::) or test(session::native_operation_unit::))'` | 12 | `i04-i06-operation-contract-verified.log` |
| `just unit-package pse-ids 'package(pse-ids) and test(owned_buffer::)'` | 17 | `i05-ownership-verified.log` |
| `just unit-package pse-relations 'package(pse-relations) and (test(columnar::integrated_performance_unit::) or test(columnar::consolidation_unit::))'` | 5 | `i05-relations-verified.log` |
| `just unit-package pse-catalog 'package(pse-catalog) and (test(delta::settlement::delta_boundary_unit::) or test(inspection::stream::delta_boundary_unit::))'` | 3 | `i05-settlement-ffi-verified.log` |
| `just unit-package pse-compiler 'package(pse-compiler) and test(/^native::tests::/)'` | 3 | `i06-compiler-verified.log` |

Interface-checked: `just check` passed for the workspace/all targets
(`i04-i06-check-verified.log`). Normal `just clippy` passed in both default and
no-default-feature modes with `-D warnings`, assessment
`build/assessment/20260920T031718.123081Z/`. `just fmt-check` passed in
`build/assessment/20260920T032203.127772Z/`. `just family-check`, `just adr-lint`
and `just docs` passed. `just py-sync` rebuilt the editable native extension and
verified its actual API stub; no Python component test was run by that build.

The first `just quality` attempt ran all 14 checks with one failure (baseline zero):
a pre-existing proptest regression seed file lacked SPDX headers. Only header
comments were added; both regression seeds were preserved. The final quality
recipe passed all 14 checks with zero failures in
`build/assessment/20260920T032338.431960Z/`
(`i04-i06-quality-final.log`). Earlier unit/Clippy failures remain in `build/plan11/`:
they exposed fixture mistakes, the stable-time classification gap, producer sharing
across cooperative wrappers and lint findings. Current receipts supersede those
results only for the named selected units and compile/static modes.

The editable extension was refreshed again after final Rust formatting with
`just py-sync` (`i04-i06-py-sync-verified.log`). `just doctor` then reported
Environment ready (`i04-i06-doctor-final.log`).

`build/plan11/i04-i06-development-checks.json` records the executed identities,
commands, modes, source hashes at execution and log digests. The mutable outcome
documents and this SPDX-only fixture edit follow the source snapshot; that drift is
not a new behavioral test run or an I17 seal. Final integration, component, parity,
solver journeys and performance remain **not run by design** until complete scope
and deletions at I17; full functional assessment is I18, measurement is I19.

### Prior I00-I03 checkpoint

Tested: targeted units with **zero failures, baseline zero**; every Rust invocation
passes `pse-relations/force-validate`. The following commands select isolated contracts,
not heater/compiler/Delta publication/solver workflows. Nextest's unselected tests are
excluded by the explicit filters, not waived failures.

| Command / exact selection | Result | Log under `build/plan11/` |
|---|---|---|
| `just setup-test` | 29 passed | `i00-script-units.log` |
| `just unit-consolidation-tools` | 13 passed | `i00-tool-units.log` |
| `just unit-package pse-ids 'package(pse-ids) and test(owned_buffer::)'` | 16 passed | `i01-ownership-units.log` |
| `just unit-package pse-schema 'package(pse-schema) and (test(resolved_contract::) or test(arrow::))'` | 14 passed | `i02-schema-units.log` |
| `just unit-package pse-relations 'package(pse-relations) and (test(integrated_performance_unit::) or test(validate::) or test(columnar::consolidation_unit::))'` | 28 passed | `i02-relations-units.log` |
| `just unit-package pse-compiler 'package(pse-compiler) and test(passes::native_construction::integrated_performance_unit::)'` | 1 passed | `i01-compiler-units.log` |
| `just unit-package pse-engine 'package(pse-engine) and (test(session::assembly::integrated_performance_unit::) or test(session::admission::) or test(session::field_transfer::) or test(session::reuse::) or test(provider::binding::) or test(session::execution::tests::) or test(session::commands::deferred::tests::))'` | 26 passed | `i03-engine-final-units.log` |

Interface-checked: normal `just clippy` covers the entire workspace/all targets in
default and no-default-feature modes with `-D warnings`: both passed in
`build/assessment/20260920T012736.146499Z/`. Normal `just fmt-check` passed in
`build/assessment/20260920T012923.740418Z/`. `just family-check`,
`just codegen-contracts-check`, `just adr-lint` and `just docs` have zero failures.
The native editable extension is refreshed with `just py-sync`; no Python component
suite is invoked by that build; final `just doctor` reports Environment ready.
The upstream `proc-macro-error2 2.0.1` future-Rust
compatibility notice is distinct from a current Clippy finding.

`just doc-lint` returned exit 2 because that command is explicitly unimplemented
under register R-20. It is **deferred**, not passed. Full integration, component,
performance, parity and distribution qualification remain **not run by design**.
No performance improvement is measured or claimed.

The existing manifest's `build/plan11/development-checks.json` records executed
binary/test identities, command modes, source hashes and log digests. No I17 seal is
created: I16-I17 and L18 are open. The I07-I12 checkpoint above
supersedes the prior continuation boundary; do not rerun historical broad campaigns.
Earlier I00-I03 receipts retain their original hashes and are historical wherever
these source changes supersede them; they are not refreshed without execution.

### I13-I15 — Write completion, retained results and Python boundary

Implemented: the [execution checkpoint](11-i13-i15-execution.md) records the actual
owner contracts, bounded scheduling, native built-ins and L03/L15-L17 replacement
receipts. Known-success commits use returned exact state; ambiguous outcomes retain
receipt recovery and publication-wide obligations. Model reuse is completed-only,
with dependency-scoped evidence and attempt-local fills. Python import uses compiled
compatibility checks while generation, quality and dynamic hook admission enforce
contract typing. The direct Arrow stream and native duplicate metadata remain intact.

Tested: 25 engine, 24 catalog and 8 runtime units using the checkpoint's exact
`just unit-package` filters with `pse-relations/force-validate`; 65 Python units using
its exact `just py-unit` selection. Baseline zero. Interface-checked: normal workspace
compilation, Clippy in both feature modes, Python quality, editable native/stub rebuild,
family consistency, pure contract generation, formatting, ADR lint and documentation.

No I17 barrier or I18/I19 qualification is claimed. The actual threaded Python consumer
case is authored; complete storage fault, solver, workflow and performance suites remain
not run until all implementation and deletion scope, including I16/I17, is complete.

### I00-I15 — Integrated implementation improvements

Implemented: H01-H12 in the [improvement checkpoint](11-implementation-improvements.md).
Shared native preparation now retains explicit relation declarations across grouped
field derivation. Actual assertion additions drive rule updates, narrow Delta misses
retain native pruning, and output groups refill completed slots before restoring
declaration order. Full source-owner, publication and local/relational evidence
boundaries remain distinct. No extra policy controls or dependencies were added.

Tested: 97 distinct Rust isolated units via the checkpoint's exact `just unit-package`
filters, with `pse-relations/force-validate`, and 32 Python cases through
`just py-unit python/pse/tests/test_transfer_contracts.py`; zero failures against
baseline zero. Interface-checked: normal workspace compilation, both Clippy feature
modes, quality, editable extension/stubs, family consistency, pure generation,
formatting and documentation. Deliberate boundaries and corrected findings are
recorded in the checkpoint. Full I18/I19 acceptance remains not run by design.

### I16/I17 — Assurance implementation and full pivot barrier

Implemented: one typed diagnostic projection crosses native/Arrow/Delta boundaries,
retaining leaf identity and aggregate multiplicity. Completion status uses the same
projection for cancellation/resources. Functional helpers default to Contract;
explicit planning-evidence tests request Diagnostic. Native DDL assertions inspect
command/effect contracts and the resulting provider constraints through wrappers.
There is no requirement for an unwrapped DDL root. Preview capture is disabled.

The generator produces 1,598 independent invariant cases and uses the discriminator's
declared enum/text field. Its pure construction unit checks both fixture variants.
The obsolete custom-scan test and 38 unused dependency declarations were removed;
macro and shared-fixture consumers remain. No dependency family pin changed.
L01-L17 remain implemented as recorded above; L18's callers are replaced.

Tested: 226 native isolated units (213 library plus 13 xtask), 66 Python units and
24 runner/audit units; zero failures against zero. Exact commands, source hashes,
selected/reported identities and logs are in `build/plan11/development-checks.json`,
`build/plan11/i17-development-native-final/`, `build/plan11/i17-python-final.xml` and
`build/plan11/i17-runner-final.log`. The previous receipt is retained separately.
The integration-only DDL assertion is compiled by normal workspace Clippy; it is not
part of the isolated library unit source set and awaits the real I18 journey.

Interface-checked: normal workspace Clippy in both modes; normal quality plus final
Python lint/types; family/overlay checks; pure contract and invariant regeneration;
ADR/index/register lint; documentation build; refreshed editable extension and stubs.
The 179 carried Plan 10 cases still resolve to existing source tests/artifact routes.
All V01-V16 rows have explicit unit/consumer/gate routes in the case manifest.
I17 seals this source only after these checks. The seal does not qualify integration
or performance; I18/I19 remain open. ADR-0074 remains proposed and the prior integrated
review remains Revise until the final assessment follows its proper lifecycle.
