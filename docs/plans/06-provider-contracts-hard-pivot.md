---
title: Provider contracts and complete native execution — hard pivot
status: abandoned
date: 2026-09-15
adrs: [ADR-0039, ADR-0040, ADR-0041, ADR-0042, ADR-0044, ADR-0045, ADR-0046, ADR-0047, ADR-0048, ADR-0050, ADR-0051, ADR-0052, ADR-0053, ADR-0054, ADR-0055, ADR-0056, ADR-0057, ADR-0058, ADR-0059, ADR-0060, ADR-0061, ADR-0062, ADR-0063, ADR-0064, ADR-0065, ADR-0066, ADR-0067]
phase: 1
---

# Provider contracts and complete native execution — hard pivot

**Execution sequence superseded on 2026-09-15 by
[Plan 07](07-unified-datafusion-delta-hard-pivot.md).** The unified DataFusion/Delta
target replaces this plan's remaining work, including its custom storage/publication
architecture and P10-only stopping point. The material below is historical scope and
evidence, not a current work queue or a requirement to preserve its runtime objects.
Plan 06 was not completed; useful functions are mapped to target outcomes in Plan 07.

## Implementation progress — 2026-09-15

**Implemented, partial:** PC00 records now include blueprint revision 38 and the
provider expansion in proposed ADR-0067. The provider map records exact 55.1.0
signatures. PC01–PC06 foundations replace duplicate inventories with one binding
index, project arbitrary native scopes, preserve actual provider owners, compose
policies, retain policies in invocation correspondence, prepare DDL before effects,
and expose owned streaming with explicit full materialization. Required invariant
IDs use the existing native diagnostic lowering. No package is marked complete by
these foundation changes.

**Implemented, additional cuts:** pass declarations now carry canonical effects;
the `executes_plans` switch and optional pass sessions are deleted. Registered stages
execute once in a native logical extension/physical operator with private output
providers. Native physical optimizer rules also run over that operator. DDL target
scopes participate before creation; quoted schema identifiers survive native SQL
lowering. Cold admission shares source parses and physical inventories through one
traversal using DataFusion's typed extension storage. `TableReader` now consumes the
common native stream with DataFusion batch splitting; the former direct snapshot
slicing path is deleted. Python retains the same native session and executor.
Catalog constructors now require the actual shared `SessionFactory`, and `Driver`
derives it from the catalog instead of accepting a second assembly. Factory policies
are inherited; cold restoration refuses omission of current factory declarations.

**Implemented, store cut:** cold manifest/ref admission, bundle publication,
completed-production publication and conditional ref updates now have prepared
native operation roots. Candidate and completed ports are captured as native
inputs; backend reads, encoding and writes execute in their bodies. Conditional
writes retain exact expected/intended content checksums and typed visibility and
durability outcomes independently of count delivery. Cancellation after a confirmed
ref update preserves completion; a lost response is explicitly indeterminate.
The invariant validator's independent session factory has been deleted. Its
callbacks now receive the actual operation session. Document and sidecar reads/writes,
ordered artifact publication, change-set publication/read/correspondence validation,
and stage-index lookup/admission/publication now execute through native operations.
Publication inputs use immutable providers; the mutable `MemTable` input projection
was removed. Nested conditional failures preserve the complete shared write journal.

**Implemented, private DML:** a captured table can bind an actual `PrivateTableFactory`.
The native planner receives a deferred operation; execution creates a distinct mutable
target, calls its real native hooks, captures the resulting rows, rechecks keys and
scoped requirements, and only then exposes the next immutable session generation.
Native created memory tables retain defaults and acquire this same private lifecycle.
The pinned memory factory supports append INSERT/DELETE/UPDATE; native unsupported
TRUNCATE/MERGE semantics remain explicit. General factories can supply their own
isolation implementations. Native EXPLAIN receives the actual deferred operation through the same hooks,
so inspection does not execute mutation. Broader command/factory/export/terminal-schema contracts remain open.

**Implemented, general capture:** any eligible native provider can enter an explicit
bounded `provider.capture` operation. Its advertised constraints are withheld during
capture; native null/aggregate checks establish primary and unique keys over the
actual complete owned rows. The resulting read-only provider retains its buffers
and exact registry owner. View dependencies must already pass source admission.
Arbitrary untagged Arrow fields retain the native type universe; PSE-tagged values
use the existing field/value validator, including nested-tag detection. Broader
nested-layout value support, facts beyond these keys and persistent capture codecs
remain part of PC03. Source model/case publication now has a native operation root;
upstream authoring/P0–P2 orchestration remains part of PC07.

**Tested — bounded receipts:**

- `just test-package pse-catalog --test provider_contracts --test native_input_ports
  --no-fail-fast --status-level fail --final-status-level fail`: **18 passed,
  0 failed/skipped**, default/force-validation/baseline 0, run
  `ecd37c9b-7ca4-4589-8119-41498d59fc2a`, 0.188 seconds. Includes native EXPLAIN under
  inspection purpose, deferred mutation hooks, retained source readers, private
  affected-row counts, rollback, defaults, and replacement obligations.
- `just test-package pse-tests-engine --test sidecar_admission --test
  catalog_store_protocol --test native_change_batches --no-fail-fast --status-level
  fail --final-status-level fail`: **18 passed, 0 failed/skipped**,
  default/force-validation/baseline 0, run
  `8d6a42da-00f0-4932-a32f-7aa1d2ecabb6`, 21.545 seconds. Includes captured read-only
  document bytes, prepare/drop without storage writes, no duplicate execution,
  sidecar/change receipts, and actual prepared stage fixtures with captured context.
- `just test-package pse-tests-lifecycle --test publication_fault_matrix --test
  local_refs --no-fail-fast --status-level fail --final-status-level fail`:
  **9 passed, 0 failed/skipped**, default/force-validation/baseline 0, run
  `71f9f5db-4365-4ae8-bd76-6cbacb464eb0`, 5.963 seconds, after the store entry-point cut.
- `just test-package pse-catalog --lib store::publication::tests --no-fail-fast
  --status-level fail --final-status-level fail`: **1 passed, 0 failed**,
  110 tests excluded by the selection, default/force-validation/baseline 0, run
  `85612b6f-b4d4-4e38-b609-ceed76c69c4f`, 0.003 seconds. A nested failure retains both
  an earlier visible write and a later indeterminate write; journal ownership releases.
- `just test-package pse-catalog --test provider_contracts --test native_input_ports
  --no-fail-fast --status-level fail --final-status-level fail`: **18 passed,
  0 failed/skipped**, default/force-validation/baseline 0, run
  `3a00d6c8-98b5-4fa7-ad70-3bb92bc3feac`, 0.171 seconds. Native private DML preserves
  old readers, reports exact counts, refuses key violations and unsupported hooks,
  and carries CREATE TABLE defaults. This predates the latest EXPLAIN hook adapter.
- **Open failed gates:** `just clippy` stopped with 74 compiler-crate errors after
  earlier catalog lint fixes; baseline 0. A prior 21-test engine selection including
  `commit_p0_p2` was interrupted after source changes made its binary obsolete:
  13 passed, 2 fixture failures, 1 interrupted test, 5 not run; run
  `67435173-32de-48be-9a05-82b4931ccb1c`, 333.577 seconds. The two storage fixtures
  now pass in the 18-test receipt above. The full source/P3 selection still requires
  a current rerun and is not certified by these storage results.
- `just test-package pse-catalog --test provider_contracts --test native_input_ports
  --no-fail-fast --status-level fail --final-status-level fail`: **15 passed,
  0 failed/skipped**, default/force-validation/baseline 0, run
  `bb133e6a-234b-4cf6-9564-c5e1b4381050`, 0.117 seconds. Includes actual immutable
  provider capture, false-PK refusal, SQL nullable-unique behavior, once-only capture
  and dependency admission before view inlining.
- `just test-package pse-tests-lifecycle --test publication_fault_matrix --test
  local_refs --no-fail-fast --status-level fail --final-status-level fail`:
  **9 passed, 0 failed/skipped**, default/force-validation/baseline 0, run
  `6a350a91-1428-4bf9-8a4d-5bc23e8c6168`, 5.918 seconds. Adds a validation callback
  that attempts to exceed its 20 MiB publication scope under a 64 MiB parent budget;
  the actual scoped allocation refuses and native invariant validation completes.
- `just test-package pse-tests-engine --test sidecar_admission --test
  catalog_pinned_reopen --no-fail-fast --status-level fail --final-status-level
  fail`: **8 passed, 0 failed/skipped**, default/force-validation/baseline 0,
  run `a38c878e-a6bc-4df9-9477-d8e42b948f44`, 7.502 seconds. The terminal fixture
  now includes the current `derivations` field; actual local predicate violations
  reject both publication and cold reopen.
- `just test-package pse-rules --test invariant_execution --no-fail-fast
  --status-level fail --final-status-level fail`: **7 passed, 0 failed/skipped**,
  default/force-validation/baseline 0, run `a6ddf36e-9983-45cc-a3f2-ab5738a21d2b`,
  6.398 seconds, after removing the invariant validator's independent factory.
- `just check`: workspace/all-targets/dev/locked, **0 errors/warnings**, baseline 0,
  after the store/validator interface cuts and before general provider capture.
- `just test-package pse-tests-lifecycle --test publication_fault_matrix --test
  local_refs --no-fail-fast --status-level fail --final-status-level fail`:
  **8 passed, 0 failed/skipped**, default/force-validation/baseline 0,
  run `7bb068e7-e496-4bbd-9414-62eead52fdf1`, 5.222 seconds. Conditions include
  independently opened local writers, exact conditional conflicts, cancelled lock
  waits, immutable-write interruption, cancellation after ref acknowledgment and
  lost-response evidence without automatic retry. This predates the validator cut.
- `just test-package pse-tests-engine --test catalog_pinned_reopen --test
  catalog_inspection --test stage_ports --no-fail-fast --status-level fail
  --final-status-level fail`: **17 passed, 0 failed/skipped**,
  default/force-validation/baseline 0, run
  `619cd1d9-4520-45dd-aa68-ccf8489c05e1`, 75.006 seconds. Native store preparation,
  once-only cold resolution/publication and retained inspection/port ownership;
  this predates the validator cut.
- `just test-package pse-catalog --lib --test provider_contracts --test
  native_input_ports --no-fail-fast --status-level fail --final-status-level fail`:
  **121 passed, 0 failed/skipped**, default/force-validation/baseline 0,
  run `7677700b-1eee-4d28-8ea9-091ebb76634c`. Shared native/platform ceilings,
  rollback on ancestor refusal, provider policies and namespace commands. Two
  test-only `unused_mut` warnings were corrected afterward; this is not a clean
  terminal compiler receipt and predates the store-publication cut.
- `just py-test python/pse/tests/test_snapshot_streams.py -q`: **18 passed,
  0 failed**, unit/component selection with xdist auto, baseline 0, 16.03 seconds,
  after successful `just py-sync` and `just doctor`. The fixture store was freshly
  built. This predates native store/validator changes; the extension needs another
  refresh before claiming current Python acceptance.
- `just test-package pse-tests-engine --test stage_ports --test terminal_attempts
  --test memo_dependencies --no-fail-fast --status-level fail --final-status-level
  fail`: **18 passed, 0 failed/skipped**, default/force-validation/baseline 0,
  run `7d1f967a-280e-45dd-bcf7-af41bcfa800f`. Conditions: actual multi-output native
  attempts, duplicate execution refusal, findings/cancellation and invocation reuse.
- `just test-package pse-rules --test invariant_execution --no-fail-fast
  --status-level fail --final-status-level fail`: **7 passed, 0 failed/skipped**,
  default/force-validation/baseline 0, run `d1bf6a28-cc8c-4c75-a3c8-cca6dc16e562`.
  Includes required PK validation before a scalar-only query over empty, valid and
  duplicate candidate inputs. The same registry lowering executes the obligation.
- `just test-package pse-tests-engine --test catalog_inspection --test stage_ports
  --test native_normalization --no-fail-fast --status-level fail --final-status-level
  fail`: **17 passed, 0 failed/skipped**, default/force-validation/baseline 0,
  run `86f385b1-711d-4682-9c59-24f938a7895c`, 344.916 seconds, one slow test.
  Conditions: native inspection and retained buffer ownership, repeated output ports,
  source normalization and cold selected-policy restoration. This run predates the
  subsequent catalog/factory constructor cut and is not a full engineering receipt.
- `just check`: workspace/all-targets/dev/locked, **0 errors/warnings**, baseline 0,
  after catalog/factory and Rust/Python consumer updates. Compilation only.

**Current next work:** finish native store/source/publication and resolution entry
points using the now-required catalog factory; complete remote/factory/function
capture, general fact admission, generated metadata and resource conformance. Refresh
the editable extension and qualify Python behavior, then finish scientific/cost
qualification and PC12 deletion/terminal/review gates. None of these receipts marks
PC00–PC12 complete. The broader operation census remains a live work list.

**Tested:** `just test-package pse-catalog --test provider_contracts --test
native_input_ports --no-fail-fast --status-level fail --final-status-level fail`,
default/force-validation/baseline 0, **9 passed, 0 failed/skipped**, run
`5f5bb9e0-8156-4f7d-8514-e0e18a9fb64e`. Conditions: local native MemTable/ViewTable,
quoted namespaces, duplicate rows, policy conflicts/settings, private DDL and stream
ownership. Later namespace declaration/rule integration edits require fresh tests.
`just codegen-bootstrap`: all three schema targets regenerated. `just adr-lint`:
67 ADR records and 31 register rows, 0 findings. Full PC11/PC12 gates remain open.

## Context

**Implementation authorized and in progress from 2026-09-15.** Implement the complete target in the
[provider-contract review](../design_review/reviews/design_review_provider-contracts_2026-09-15.md),
including all nine findings, every standardization boundary, and its PR00–PR09
packets. This plan replaces the remaining execution instructions in
[Plan 05](05-native-logical-plan-hard-pivot.md) and its
[restart handoff](05-native-logical-plan-hard-pivot-restart.md). Their useful domain
requirements and historical receipts remain inputs. They are not another work queue.
The review's instruction to rebase Plan 05 is fulfilled here by one successor plan.

The maintainer's 2026-09-15 direction is a design-phase hard pivot. Build the target
directly. Reuse an existing declaration, algorithm, owner or storage primitive only
when it contributes to that target. Delete replaced code, callers, data objects,
development stores and acceptance machinery with their replacement. Git history
is sufficient for historical source. There is no compatibility API, dual engine,
fallback flag, predecessor graph requirement or old-versus-new qualification work.

One maintainer works through Codex in one checkout and one implementation stream.
The packages below organize dependencies; they do not require separate branches,
agents, PRs or approval rounds. A bounded replacement may temporarily break its
callers. Replace those callers as part of the same cut, then restore the affected
build; do not add compatibility shims merely to keep intermediate commits green.

### Authority and evidence

- The blueprint remains architecture authority: D1–D14, §3.3.3, §4–§8, §6.15,
  §14, §18, §20–§24. PC00 records the expanded provider contract before dependent
  implementation. The listed ADRs identify inherited contracts, including proposed
  ADR-0067; listing them does not imply their statuses have changed.
- The review is design evidence, with **G7 failed and G1–G6 unresolved** for this
  expanded target. Its §§2–4 supply the target; §§7, 9–11 supply findings, tests
  and required policy changes. No gate is closed by writing this plan.
- **Interface-checked, review evidence:** DataFusion 55.1.0, Arrow 59.3.0 and
  Rust 1.98.1 were checked against the resolved dependency graph and installed
  sources. See the [context record](../design_review/evidence/provider-contracts-context-2026-09-15.json)
  and the review's exact-release corrections. The
  [provider map](../capability-maps/datafusion_provider_contracts.md),
  [logical-planning map](../capability-maps/datafusion_logical_planning_capability_spec.md)
  and [semantic/UDF map](../capability-maps/datafusion_semantic_analysis_types_udfs_capability_spec.md)
  are discovery inputs, not substitutes for the pinned contract.
- **Tested, characterization only:** the [five review probes](../design_review/evidence/provider-contracts-probes-2026-09-15.md)
  passed with zero failures/skips/warnings, default/force-validation/baseline 0.
  Three assert current defects. Replace their expectations with target conformance
  tests; do not preserve the defects to keep these probes green.
- Use Context7 for new library questions, then pinned source/rustdoc, Cargo metadata
  and a focused executable probe for a load-bearing uncertainty. Retain reproducible
  evidence. Do not repeat the completed broad discovery or survey unused APIs.

### Current boundary to carry forward

The native-plan pivot already supplies useful generated Arrow access, semantic
UDFs, exact input ports, private producer completion, conditional storage, source
fixtures and owned exports. Several legacy graph/replay/row paths and golden stores
are already deleted. Preserve those deletions and adapt useful target mechanisms.
PC00 records the live inventory; an old symbol in a deletion table does not require
recreating or rediscovering it.

The latest engineering-inspection attempt failed on the P3 `packages` input-role
collision. P0's internal role was renamed to `package_dependency_headers` and its
focused regression passed: `just test-package pse-authoring --test p0 --no-fail-fast
--status-level fail --final-status-level fail`, default/force-validation/baseline 0,
**2 passed, 0 failed/skipped**, run `f91e8775-c782-4f6b-aa94-66a94bffb710`.
The complete engineering workflow and Python inspection were **not rerun after
that fix**. Earlier four-case engineering success predates later producer/port
changes. Neither that receipt nor the five review probes establishes current
product acceptance. Continue with the foundation replacement, not another series
of local name patches.

At this planning check, `just doctor` reported **2 blocking issues, baseline 0**:
installed UV 0.12.15 versus the required 0.12.13, and the resulting environment
check failure. PC00 repairs the needed tooling through the bootstrap workflow;
refresh the target editable extension in PC10. No environment repair or native
implementation is claimed by this documentation task.

## Decisions

### Delivery and scope

| Required outcome | Delivery boundary |
|---|---|
| Universal operation framework | Every callable product read, transformation, diagnostic, mutation, compile, import, reopen, export and execution enters a bound provider/native-operation context |
| Full hierarchy | `CatalogProviderList`, `CatalogProvider`, `SchemaProvider`, `TableProvider`, factories, views and table functions share binding, naming, policy, metadata and lifecycle rules |
| Native planning and execution | One effective native session, complete function/planner eligibility, native plans/operators/sources/sinks, owned streams and explicit effects |
| Real engineering product | Source documents → P0–P2/authored model → actual P3–P10 → persisted indexed `CanonicalMathGraph` → cold Rust/Python inspection |
| Physical breadth | Required units/elements/quantity algebra, ideal properties, FTPx/FcTP states, steady flowsheet, lumped control volume, feed/product/heater/mixer/state junction and equality connections |
| Methods and laws | Sourced NIST Shomate and RPP4 cp/h/s, required Perry liquid cp/h/s and density; componentTotal/componentPhase/elementTotal, enthalpyTotal/isothermal, pressureTotal and declared mixer total-flow conservation |
| Existing specialized operations | Quantity/MathIR, numerical kernels, solver/backend and other currently callable data operations get contracted native entry points and results; ordinary inner algorithms remain reusable |
| Later product breadth | P11–P14, solve-ready problem construction, new solver functionality, Pyomo generation and distributed execution remain later delivery scope. This does not exempt any already callable operation from the framework |

No product-operation exception is accepted. Generation/bootstrap tooling can run
before a runtime exists because it constructs that runtime's declarations. FFI
callbacks, numerical loops and OS primitives are internals of bound operations.
An unimplemented future feature has a truthful unsupported outcome, not a bypass.
All DataFusion/Arrow functionality remains eligible; purpose-specific semantic and
effect requirements govern particular invocations. No confined UDF list returns.

### One authority, native interfaces

Extend existing declarations and assembly rather than building a second provider
language or dispatch registry. The names below describe responsibilities, not a
requirement to introduce this exact list of Rust types.

| Responsibility | Canonical owner | Bound or generated projection |
|---|---|---|
| Relation, operation/pass, role, policy, invariant and diagnostic meaning | Existing `pse-schema` models and authoritative policy/reference relations | Rust/Python contracts, Arrow fields, obligation inputs, metadata schemas |
| Actual implementation | Assembly registration of the real provider/function/planner/operator/codec with its declared contract | Retained implementation objects and supported-operation bindings; descriptive capability rows |
| Resolution generation | One catalog-owned inventory of exact source revisions, scoped names, role instances, coverage and actual owners | Native hierarchy, `TableSource`, typed algorithm inputs, metadata and inspection lookup |
| Effective policy | One composition of participating root/catalog/schema/table/invocation declarations | Immutable native configuration, semantic/effect requirements and policy-origin views |
| Preparation | Catalog computation/session layer below compiler/rules consumers | Native plan, captured dependencies, output ports, known facts and residual obligations |
| Execution | Native operators under the shared runtime | Fresh attempt workspace, streams, observations and private typed outputs |
| Admission and publication | Common completed-obligation ownership and existing conditional storage protocol | Complete admitted bundle, manifest/ref transition, explicit outcome |

Catalog must not import compiler/rules to discover implementations. Higher-level
assembly supplies native operations and prepared obligations through the shared
contract. Rule/invariant lowering has one implementation; catalog does not add a
second PK/FK/domain interpreter. Keep the current acyclic crate layering unless a
concrete target dependency problem requires a recorded change. No new crate or
dependency-family upgrade is assumed.

A table binding contains the declared relation version, scoped instance/input or
output role, source revision/encoding, admission state, actual implementation,
dependency scope and retained resource/data owners. Multiple bindings may share
immutable data while remaining semantically distinct. Names locate these bindings;
they do not create semantic identity. Resolve `head` and other mutable aliases once
per operation. Do not infer exact source identity from an alias or snapshot ID alone.

### Standardization at each level

| Level | Required standardized behavior | Owning packages |
|---|---|---|
| Root list | Normalized name keys, captured default catalog, alias/generation selection, visibility/coverage, faithful insertion and replacement | PC01–PC02 |
| Catalog | Schema ownership, generated scopes, replacement, cascade policy, deterministic promised enumeration, metadata budgets | PC01–PC02 |
| Schema | Atomic table insert-if-absent, duplicate refusal, owner/type metadata, existence/lookup coherence, coverage and failure semantics | PC02–PC03 |
| Table metadata | Declared schema/defaults, table/view kind, established constraints, statistics precision, actual provenance and admission state | PC03, PC05 |
| Scan planning | One `scan`/`scan_with_args` planner; common predicate classification/translation; projection, limit, statistics and plan-property correctness | PC05 |
| Table mutation | Real INSERT/DELETE/UPDATE/TRUNCATE/MERGE support bindings, base generation, candidate isolation, row accounting and completion | PC06 |
| Factories | Typed options, physical versus semantic schema, source/revision, declared discovery/effects, resources and implementation version | PC03–PC04, PC06 |
| Views/table functions | Parameters, exact input generation, schema/meaning, dependency and effect binding before inlining; cheap provider construction | PC03–PC04 |
| Native session/planners | One combined policy, actual function/planner/codec owners, preparation and command admission including plans without scans | PC04 |
| ExecutionPlan/DataSource | Schema, order/partitioning/boundedness, fresh state, cancellation, accounting, spill, late failure and observation | PC05–PC06 |
| DataSink/publication | Complete output artifacts, bundle obligations, conditional visibility, retry and durability outcomes | PC06 |
| Rust/Python surfaces | Same lookup, metadata, policy refusals, plans and owned streams; read-only inspection after explicit admission | PC10 |

Native traits do not supply policy inheritance, domain validation or transactions.
Implement these shared rules once at their proper operation/session boundaries.
Do not attach an analyzer to every schema or require a scan callback to govern a
scalar-only query. Downcasts may adapt an implementation or aid diagnostics; they
must not select business policy from a closed list of concrete provider types.

### Policy composition and native mutability

1. Semantic requirements conjoin. Established facts retain exact owners and
   premises; narrower scopes cannot mark an invalid candidate valid.
2. Effective effects/support combine purpose, inherited restrictions and actual
   operation implementations. Read, external observation, nondeterminism, namespace
   mutation and durable publication are separate dimensions. Metadata booleans
   cannot supply missing implementations.
3. Overridable defaults use declared root → catalog → schema → table → invocation
   precedence. Required semantic settings are not ordinary defaults. Normalize SQL
   identifiers once; retain quoted identity. Cross-catalog conflicts require an
   explicit compatible selection/conversion or a typed refusal.
4. All scopes charge one runtime. Child budgets cannot exceed selected parent
   budgets; thread/partition/batch/spill settings form one execution configuration.
5. Generate effective-policy, origin, capability, coverage and dependency views
   from those same owners. They are explanations, not a second configuration API.

`CatalogProviderList::register_catalog` has no error channel and must insert or
replace faithfully. Use a conforming list in private assembly/command state and
fallible product commands for mutation policy. Prepared reads retain selected
bindings. Native extensions see attempt-scoped namespace state; registration cannot
change a published generation or another prepared operation. Record generation
changes and reject undeclared effects before output admission/publication. Never
return `None` as a pretend rejection or claim a raw trait object is an immutable
capability boundary. Prefer a small generation overlay sharing immutable owners;
do not rebuild every catalog and session for each added role.

### Preparation, execution and publication

```text
declarations + actual source/implementation owners
    → explicit resolution/admission + complete provider generation
    → captured policy, dependencies, parameters and output contracts
    → native construction, analysis, semantic obligations, optimization
    → immutable prepared operation
    → fresh attempt: native streams/operators + private output catalog
    → complete required outputs and obligations
    → admitted result → explicit conditional publication, when requested
```

DDL must first become a logical command. Admit its effects and bind its attempt
before calling an upstream handler that may mutate during `sql()`. EXPLAIN,
prepare/drop and rejected commands do not perform mutation. Explicit metadata
resolution may do declared I/O; planning an in-memory read performs no row execution.

One multi-output operation owns typed output ports in its private catalog. Its
native execution stream reports its outcome; it does not flatten all relations
into an untyped universal batch. Only completion exposes the actual output owners.
Reading an output or its metadata does not rerun the producer. DML's native UInt64
`count` is affected-row data; application completion/publication records determine
whether a change became durable. A stream batch or count alone cannot publish.

Keep partial, failed, cancelled, complete/admitted, stale-base, published and
visible-but-durability-uncertain outcomes distinct. Repeated execution uses fresh
physical state. A retried logical command must reconcile its intended ref/outcome;
do not claim exactly-once behavior from a consumed stream or a count row.

### Facts, resolution and ownership

- Physically safe candidates may be queried without advertising PK/unique facts.
  Row-local pure checks/semantic retention may use reusable UDFs. Keys, foreign
  keys, coverage, source correspondence and coupled outputs require complete
  relation/bundle obligations or sound constructive facts. Effects are explicit
  commands, not side effects disguised as validation UDFs.
- Preserve field/domain/quantity/refinement meaning through scan, projection,
  filter, joins, union/distinct, aggregate, window/unnest, casts/functions and
  recursion. Use native schema/FD/property facilities where sufficient. Lost
  key/coverage premises remove their dependent guarantees. No detached metadata
  repair invents a proof or a unit conversion.
- Resolve remote metadata explicitly before exposing cheap synchronous lookup.
  Retain backend revision support or declared live-observation semantics, full
  versus referenced-subset coverage and negative-read scopes. Local caches do not
  prove a coherent remote snapshot. Implement a local resolver plus adversarial
  fake backend first; no speculative remote service is required.
- Capture dependencies before optimization: role/source, aliases resolved to exact
  owners, absent/empty inputs, complete candidate universes, defaults/shadowing,
  definitions, parameters, views, functions, codecs, policy and external reads.
  Folding/inlining must not erase their premises. Hashes locate reuse candidates;
  actual context and required admission establish eligibility.
- Stream by default. Materialize deliberately for shared outputs, whole-bundle
  obligations, persistence or a foreign/algorithm boundary. Share immutable parsed,
  physical, metadata and admitted owners. Pending work is wakeably cancelled;
  retained exported arrays retain their leases after the handle closes.

## Plan

Package acceptance remains **Proposed** until its recorded exit checks pass. Execute top to bottom in one stream. The dependencies
identify required mechanisms, not opportunities for parallel agent choreography.
Each package includes replacement and deletion; PC12 audits the remainder.

| Order | Package | Depends on | Review packet |
|---|---|---|---|
| PC00 | Record target contracts and scope census | — | PR00 |
| PC01 | Declare bindings, policy and generated projections | PC00 | PR01 |
| PC02 | Replace the hierarchy and resolution generation | PC01 | PR02 |
| PC03 | Generalize admission and metadata | PC02 | PR03 |
| PC04 | Bind all preparation, functions and effects | PC03 | PR04 |
| PC05 | Standardize scans, streaming and runtime ownership | PC04 | PR05 |
| PC06 | Implement commands, multi-output completion and publication | PC04–PC05 | PR06 |
| PC07 | Cut source authoring and P0–P3 onto native operations | PC06 | PR07 |
| PC08 | Complete rule, reference and specialized operation bindings | PC05–PC07 | PR07 |
| PC09 | Complete P4–P10, cold admission and exact reuse | PC07–PC08 | PR07 |
| PC10 | Unify Rust/Python inspection and extension journeys | PC03, PC05, PC09 | PR08 |
| PC11 | Establish target product and cost evidence | PC09–PC10 | PR09 |
| PC12 | Delete remainder and close every gate | PC11 | PR09 |

**Execution strategy:** PC02–PC04 replace the source of the current integration
failures before more compiler repair. Use one real source-bound read and one
multi-output P3 operation as foundation consumers through PC05–PC07. Extend the
same route through P10 and cold inspection. Do not construct a generic provider
platform first and postpone all domain consumers until the end. When a foundation
signature changes, delete or update its affected caller in that cut; later packets
finish domain breadth rather than maintain an old callable route.

### PC00 — record target contracts and scope census

**Owns:** proposed ADR-0067, affected blueprint sections through the explicit design
revision workflow, provider capability map, active rules/docs, this plan's inventory.

1. Recheck ADR-0067's status. While proposed, extend that coherent native-plan
   decision with provider generations, purposes, policy composition, faithful root
   mutation, native commands, multi-output ownership and inspection. If it has
   become immutable, allocate a scoped successor using `just adr-new`; never edit
   an accepted argument or preassign a number. Record review findings and the
   required independent G1–G7 assessment without marking unresolved gates accepted.
2. Amend blueprint §3.3.3, §5.4, §14.3/§14.3.1, §20 and §22 as affected, preserving
   stable section IDs and adding the revision row/decision references. Use
   `PSE_DESIGN_EDIT=1` openly for that work. Follow the ADR/design PR status rules;
   they do not require a new eligibility decision or a PR per implementation packet.
3. Reconcile the provider map's exact 55.1.0 signatures and semantics: installed
   `scan` projection is `Option<&Vec<usize>>`; structured projection is a slice;
   no preferred-ordering field; root has no typed rejection; async resolution is
   sequential/subset caching by default; constraints do not validate; statistics
   requests are optional; view construction does not validate and can inline;
   DML counts are not commit receipts. Replace independently editable capability
   sidecars with projections of actual bindings. Correct fixed min/max claims.
4. Inventory current Rust exports, Python methods, CLI/xtask product operations,
   direct session/scan/collect/store calls and compiler/solver entry points. Map
   every callable operation to the operation table below and a package. Distinguish
   build tooling from product operations. Record target-useful pieces and deletions
   already completed; do not requalify the discarded architecture.
5. Remove active seven-schema ceilings, snapshot-only session assumptions, closed
   provider/UDF rosters and mandatory replay rules that conflict with this design.
   Keep type-universe, generated-source, clean-room, identity and publication
   contracts. Repair needed bootstrap tools without changing pins to match an
   accidentally installed version.

**Exit:** one explicit authority/contract map and complete callable-operation census;
no policy blocks provider adoption. Any remaining scientific declaration belongs
to PC08/PC09. `just adr-lint`, affected governance checks and documentation checks
verify the revised records. No implementation acceptance is inferred from them.

### PC01 — declare bindings, policy and generated projections

**Owns:** `pse-schema/src/model`, its generators, catalog binding/computation types
and shared runtime configuration contracts; generated output only through codegen.

Extend relation/pass/operation declarations with scoped input/output roles, purposes,
effects and dependency obligations. Define one relation-instance identity and
one name-resolution policy that cover all versions, repeated ports, candidates,
attempts and metadata. Separate required semantics, actual supported operations,
established facts and defaults. Generate policy-origin/capability/coverage/outcome
schemas and Rust/Python projections; do not create another handwritten manifest or
DTO family. Generate exact diagnostics for absence, unsupported, conflict, invalid,
partial and uncertain outcomes from their existing authoritative declarations.

Bind actual implementation objects when assembling declarations. A descriptive
capability record is derived from those registrations and confirmed by method
conformance; callers cannot mint trust through a flag. Keep private constructors
for established facts and completion. Reuse generated typed Arrow views/builders,
including nested/dictionary types and strict current codecs; finish any remaining
mandatory per-row Cell serialization/admitted-view rescan removal from HP01.

Implement policy composition once, including cross-catalog incompatibility,
non-overridable semantic requirements and child resource limits. Runtime
configuration extensions project this result rather than become a competing source.

**Delete:** duplicate binding/default/policy declarations and generated artifacts
owned only by retired APIs. **Exit:** codegen equivalence, binding identity cases,
policy conflict/precedence tests and forged-fact refusal pass. Bootstrap generation
starts no DataFusion session and no new dependency cycle is introduced.

### PC02 — replace the hierarchy and resolution generation

**Owns:** `pse-catalog/src/provider/{list,catalog,schema}.rs`,
`session/{roles,snapshot_session}.rs` and catalog resolver/binding assembly.

Build one scoped table-entry inventory. Generate all domain, operation, input,
candidate, output and metadata catalog/schema entries from it. Before/after inputs,
two versions and zero-row inputs retain exact independent roles. Internal functions
receive scoped binding handles; they no longer invent strings such as `packages`
or prefixes to avoid collisions. SQL references, native direct scans, typed
algorithm projections and metadata derive from the same provider entry.

Use native memory registry implementations where their behavior fits; implement
only missing generation/policy glue. Share map mechanics without erasing native
replacement versus duplicate-refusal differences. Test atomic insert/update
behavior, deterministic promised names, removal/cascade, quoted names and default
catalog/schema resolution. Implement faithful root registration in private state,
immutable captured generations and explicit generation changes for commands.

Add explicit resolver coverage and failure states. Cheap lookup reads resolved
metadata; `None` means absent within that coverage. Native signatures that cannot
return a backend error receive only completed resolution. Test a fake changing
backend with revision capture, referenced subsets, duplicate/negative references,
failure, cancellation and bounded concurrency. A backend without coherent revision
support must select explicit observation/capture semantics or refuse the requested
reproducibility guarantee.

**Delete:** independently meaningful source/role/computation maps, repeated catalog
reconstruction on every role, duplicate name canonicalizers and the root no-op.
Indexes or immutable overlays derived from the inventory may remain.
**Exit:** positive replacements for the role and registration probes pass; the same
owner is visible through SQL/native/metadata, and retained prepared readers survive
later namespace changes without rebinding.

### PC03 — generalize admission and metadata

**Owns:** `session/admission.rs`, provider/table registration, semantic contract
admission and information-schema/PSE metadata providers.

Replace the `RelationTable`/`CandidateTable`/`ComputedTable`/`IndexedTable` roster
with actual bound implementation contracts. General native providers, views,
table-function results, native worktables and file/factory sources use this entry.
Direct physical/native access must retain the same binding obligations. No
`trusted=true` token, unrestricted raw batch substitution or speculative mutation
probe establishes support or validity.

An actual provider object alone does not establish immutable data. Mutable external
backing must provide a version-bound read or be captured/isolated before enduring
facts attach to it. Otherwise retain candidate/live-observation semantics and refuse
unsupported snapshot claims. Prepared readers cannot share a mutable table instance
whose later changes invalidate their captured facts.

Project table schema, kind, definition/default expressions, owner, established
keys, supported operations and provenance from bound owners. Candidates expose
the schema needed to validate them but withhold the constraint under examination.
Generate whole-relation and cross-port residual obligations through the shared
lowering implementation. Extend the existing semantic transfer rules for actual
view/join/aggregate/union/unnest consumers; preserve exact equality, nullability,
quantity and occurrence meaning. Unknown facts remain unknown until established.

Admit ordinary native `information_schema` through the same metadata binding path.
Add generated PSE views for semantic facts, dependencies, effective policies and
origins, capability support, coverage, operation/attempt outcomes and observations.
Cheap table/type lookup does not execute rows. Columns/default/view metadata can
resolve providers under an explicit metadata budget; do not promise it is free.
Unavailable metadata or failed resolution is never an empty relation.

**Delete:** the concrete admission roster, fixed settings-query bypass, duplicate
PK/FK/domain predicates and editable capability booleans. **Exit:** a native view,
table function, file provider and custom diagnostic provider enter without another
admission branch. Forged constraints, wrong owner/context, duplicate candidates and
missing coverage fail with the expected diagnostics; ordinary metadata succeeds.

### PC04 — bind all preparation, functions and effects

**Owns:** `session/{factory,preparation,functions,profile,physical_fields}.rs`,
native plan/codec boundaries and common operation preparation.

Assemble the actual native `SessionState` once for effective policy and retained
bindings. Preserve normal analyzer, logical and physical optimizer pipelines plus
PSE semantic rules. A custom `Session` adapter delegates the required real behavior;
do not accidentally accept identity optimization, empty physical rules or an
unsupported default planner. Share immutable owners when forking; execution state
is fresh even when a prepared logical computation is reused.

Capture source, role, view/table-function, scalar/aggregate/window/higher-order
function, planner, codec, settings, default and external-read dependencies during
binding before lowering/folding/inlining. Apply purpose/effect admission to every
plan, including scalar-only, empty, extension and command plans. Keep stable,
volatile, async and configuration-aware functions eligible: reproducible compilation
binds their relevant inputs or refuses the unbound effect; exploratory operations
can select declared observation semantics. Unknown custom effects cannot be assumed
pure just because a provider scan disappears.

Build SQL logical plans before effectful dispatch. Inspectable preparations expose
actual original/analyzed/optimized plans and obligations. Do not use eager DDL APIs
as a parser. Native mutation handlers are invoked only after PC06 binds an admitted
attempt. Serialized plans are optional diagnostic/transport mechanisms, not memo or
semantic identity; where an actual consumer decodes them, require its bound codecs,
sources and implementation context and test that supported route.

**Delete:** per-provider policy interpreters, preparation bypasses, narrow engine
profiles and detached field-repair assumptions. Keep pure reusable semantic UDFs
where sound. **Exit:** inlined views, folded/zero-row/scalar-only plans and custom
nodes retain the required obligations/effects/dependencies. EXPLAIN/prepare/drop
execute no in-memory row kernels or mutation. Conflicting contexts and hidden
implementation changes fail or invalidate the correct preparation.

### PC05 — standardize scans, streaming and runtime ownership

**Owns:** `provider/{table,pushdown,statistics}.rs`, candidate/computed/indexed
sources, `session/preparation.rs`, `pse-runtime` and native allocation/stream adapters.

Route `scan` and `scan_with_args` through one scan planner. Predicate classification
and translation share one implementation, with native residual filtering for
unsupported/inexact portions. Cover filter-only columns, repeated/reordered/empty
projections, exact field metadata, nulls, duplicate rows and limit ordering. Merge
common mechanics where semantics match; retain the meaningful candidate/admitted
distinction. Use native DataSource/file machinery where applicable and keep row
work in execution, not physical planning.

Supply truthful partitioning, ordering, boundedness, row counts and other requested
statistics with explicit exact/inexact/unknown precision. Cache statistics by the
same immutable source generation. Implement useful selected min/max/null counts
when needed; otherwise report unavailable and reconcile the prose. No invented
statistics or unsupported hints may justify optimizer pruning. Keep bag semantics
correct, including the previously identified ALL-set lowering issue at this pin.

Return owned streams; remove unconditional `Vec<RecordBatch>` collection and
workspace recreation. Deliberate materialization records why it is needed and
retains one owner across consumers. Cancellation wakes a pending stream and stops
owned tasks; failure/drop releases private allocations. Share one DataFusion
memory/spill/runtime context with explicit PSE leases and configurable budgets.
Do not claim accounting for arbitrary process allocations that the runtime cannot
observe. Replace stale tiny object/row limits with typed settings and real format
limits, while retaining intentional refusal tests.

**Delete:** duplicate scan/filter interpreters, bulk planning-time evaluation,
automatic collect/copy wrappers and hidden per-provider pools.
**Exit:** V05/V10 below pass, including a deliberately over-pruning provider caught
by the oracle, partition/batching variation, repeated execution and retained-array
ownership. Early instrumentation records first/last batch and actual materialization.

### PC06 — implement commands, completion and publication

**Owns:** catalog computation/command handling, mutation providers, factories/sinks,
`store/{changes,control,manifest,membership,open}.rs` and attempt/terminal contracts.

Implement read, candidate, authoring, execution and publication purposes in one
operation lifecycle. Native DDL, session-setting/namespace commands, Copy/export,
external-table factories and table DML enter through admitted logical commands.
For each actual PSE mutable table, bind INSERT/DELETE/UPDATE/TRUNCATE/MERGE to its
real supported semantics; unsupported source/operation combinations return the
standard truthful diagnostic. Test defaults, exact affected-row accounting,
conflicting/ambiguous matches and schema/constraint failures. Do not treat an
external inferred schema as an admitted semantic declaration.

Create one private attempt catalog and explicit output ports for multi-relation
commands and compilation. Native operators execute actual implementations; one
operation owns its output completion. Output scans cannot rerun it, and consumers
cannot assemble a successful bundle from arbitrary batches. Validate residual
field, relation, cross-port and source-correspondence obligations over that bundle.
Native DML count streams and application outcomes must describe the same attempt.
The command root runs once per attempt across partitions and consumers; duplicate
stream opens cannot apply the command again. Keep intermediate counts private where
a publication command requires a final commit outcome before reporting success.

Reuse current conditional object/ref primitives behind a DataSink or explicit
publication ExecutionPlan. Stage complete artifacts and manifest before the ref
transition. Record expected base, intended result, command identity and attempt
outcome; test stale base, late failure, cancellation, ref conflict, lost response,
visible-but-uncertain durability and retry/reconciliation. A session namespace
change and a durable model publication are separately declared effects. Observe
undeclared extension-side namespace changes and refuse admission/publication.

Generate one terminal record/error schema. Recording failure remains explicit
alongside the original failure; a later auxiliary index failure must not rewrite
an already committed success. Fresh execution resets/rebuilds physical state;
retries must not silently apply a committed logical edit twice.

**Delete:** separate successful producer-result assembly, mutation validators,
publication decisions and bare command bypasses once the common route owns them.
**Exit:** V07/V08 pass with a real two-output operation, all native DML hooks on
appropriate target tables, namespace/factory/Copy routes and injected publication
failures. No partial port or failed obligation becomes authoritatively visible.

### PC07 — cut source authoring and P0–P3 onto native operations

**Owns:** `pse-authoring/src/{document,p0,p1,change_set,targets}`,
compiler `passes/p2.rs`, Driver/P3 inputs and configuration, source providers and
schema evolution.

Reuse the grammar/AST and one owned parse per changed document. Source providers
retain actual bytes, lexical scope, spans, fields/keys and resolved semantic IDs.
Native joins bind owners, references, prospective instances and typed selectors
with exact zero/multiple-match findings. P0 dependency closure uses the complete
package universe, duplicate/missing/pin checks and an explicit cycle/depth algorithm.
P1/P2 use the common binding and admission program. No fixed internal-role strings
share the namespace of caller ports.

Replace change/rename orchestration with bound before/after providers and ordered
native change plans. Preserve exact preimages, operation ordinals, explicit-ID
rename and lexical shadowing; repeated edits of one key are not collapsed silently.
Reparse changed bytes once and reuse unchanged parse owners. Current declared
schema evolution uses selected native projections/conversions and residual
obligations; no automatic pre-pivot store migration is introduced.

Run the real P3 as the first complete source-authored multi-output operation on
PC06. Native normalization/seed expansion consumes cached configuration by exact
identity/ordinal; AST, quantity and MathIR construction remain contracted algorithms.
Use the existing source fixtures and real Driver, replacing their orchestration
where needed. Do not fabricate P3 output to validate downstream wiring.

**Delete:** parallel source inventories, repeated decode/rebind/proof copies,
verification-only ChangeSets, producer replay after local completion and source
dispatch wrappers. **Exit:** a fresh authored source and ordered edit/rename produce
the exact P0–P3 bindings/ports and errors; parse/producer counters establish once
per required local computation. The actual engineering case passes its P3 boundary.

### PC08 — complete rule, reference and specialized operation bindings

**Owns:** `pse-rules`, reference packages/generators, quantity/material/MathIR
adapters and existing numerical/kernel/solver/backend entry points from PC00.

Bind rule execution to native worktables in the same attempt. Facts, deltas,
unknowns, conflicts, support edges and settled negative scopes are typed relations.
Use native joins/aggregates/windows/differences and a small fixed-point driver for
finite positive closure. Truth identity differs from support identity; several
independent supports remain represented. Whole-stratum conflict decisions settle
before downstream use. SQL null does not silently define four-valued rule truth.
Exhaustion/cancellation is incomplete, not convergence. Recompute affected complete
strata on deletions/support loss; do not add a fine-grained maintenance project or
use support counts alone to preserve an unsupported recursive cycle.

Complete the delivery's reference packages from one sourced declaration set,
including the named state formulations, laws and methods in Decisions. Generate
leaf/test projections; independently establish dimensions, affine/reference
conversions, composition basis, formula parameters/domain and natural coordinates.
Source evidence and independent physical values are required; generated agreement
and an IDAES code copy are not substitutes. Keep clean-room boundaries intact.

Share one admitted quantity/material inventory per bound generation. Use native
plans for lookup/composition and graph selection; preserve exact rational/affine
algebra and D6 indexed MathIR as specialized implementations with typed inputs,
ordered roots/arguments, payloads, binders and occurrence/source provenance.
Wrap each existing callable numerical/solver/backend operation in the same native
invocation/resource/result contract. A whole coupled solver problem cannot be
changed by ordinary analytical projection/filter pushdown. Keep callbacks internal;
supporting new solve-ready product breadth is outside this delivery.

**Delete:** duplicate row/literal fixed-point engines, repeated reference/physical
decoders, hand-maintained standard defaults and direct product-level algorithm
entry points that bypass the common operation. **Exit:** V11/V12 pass; one new
rule/diagnostic or template using existing constructs is declaration-driven, and
all presently callable specialized operations have actual bindings or explicitly
declared unsupported states rather than alternative execution paths.

### PC09 — complete P4–P10, cold admission and exact reuse

**Owns:** compiler P4–P10/Driver/memo/validator, templates, catalog
`store/{pinned,invocation,stage_admission}` and shared traversal context.

| Consumer | Required target behavior |
|---|---|
| P4 | Inherited dynamic/holdup settings, typed implications/exclusions, useDefault balances, phase/species restrictions and actual method compatibility; no dependency on future P7 output |
| P5 | Finite instances/scopes, containment, selectors/port members, topology and boundary cuts; invalid containment differs from physical recycles; deterministic heuristic tear selection actually breaks the intended cycles |
| P6 | Complete equation/law/port/display/initializer/transitive demands and candidate universe; finite settled resolution with all requesters and positive/negative support; no late producer demand silently restarts compilation |
| P7 | Generic parameter/domain binding, guards, aliases, indexed symbols/equations and contributions; declarations create heater/mixer physics; preserve occurrence identity despite shared expressions |
| P8 | Complete inclusion/exclusion reasons, actual law participation and internal-transfer cancellation; symbolic residuals with declared arithmetic/guards; indexed equality connections and mixer conservation rather than inlet-temperature equalities |
| P9 | Actual selected method provisions, ordered parameters, single natural-unit conversion, missing/undeclared requirement errors; exact kernel signature/domain/outcome/null/error/derivative binding and honest backend availability |
| P10 | Real P9 plus complete physical/domain/reference/kernel context; acyclicity, typing, constants/conversions, quantity/shape, ordered roots/arguments and canonical indexed graph with full derivations; no early case binding |

All pass inputs and completed ports resolve through the provider generation.
Driver owns sequencing/attempts through native operations; no second opaque
producer dispatcher remains. Reuse target-useful current graph inputs; delete any
predecessor object retained only for historical validation. Record empty output
ports explicitly. No unit-name dispatch or fixture-only P10 route closes this work.

Implement current-format cold open/import as an explicit read-only
resolution/admission operation. A traversal-scoped owner shares admitted parents,
original parses, physical inventories, implementations and policies. Establish actual
artifact properties through the same obligations. If durable source/derived
correspondence requires computation, invoke the same current native producer once
per distinct required binding within that admission traversal. Do not rerun each
stage independently, trust a saved `valid` marker, default missing policy, repair
missing stages or publish during open. Only return the admitted handle after this
declared boundary completes; subsequent lookup never compiles or re-admits.

Complete-stage reuse compares exact admitted inputs, roles, implementations,
configuration, source/negative scopes and current admission requirements. Bind
the original settings and parent encodings from the current generated manifest/
invocation format. Warm reuse shares completed owners; restart reuse performs its
declared current-format admission. Each execution/reuse request retains its proper
attempt record. Alias spelling and plan serialization are not semantic authority.

**Delete:** hardcoded producer/admission replay rosters, per-stage context rebuilds,
default-policy import fallback, duplicate result adapters and obsolete memo/wire
formats. **Exit:** real heater/mixer × FTPx/FcTP closes P0–P10, publishes and cold
opens in Rust; V09/V13 and exact memo invalidation pass, including changed absence,
provider, policy, default and source-support cases. Negative scientific/provenance
expectations pass alongside the positive graph tests.

### PC10 — unify Rust/Python inspection and extension journeys

**Owns:** `pse-catalog/src/inspection.rs`, `pse-py`, `python/pse`, generated boundary
contracts, `xtask` inspection helpers and current API/example documentation.

Replace direct snapshot membership lookup and loaded-relation slicing with common
binding resolution and provider-backed owned streams. Keep immutable admitted
handles and the existing array lease guarantees. Expose the same metadata, native
plan/operation explanations, purpose/capability errors and observations in Rust,
SQL and Python. Generated projections use the one contract; Python has no separate
policy implementation. Preserve pinned pyo3-arrow and Python annotation/import
requirements. Physical export support does not imply consumer extension support.

Define close, partial drain, cancellation, exception and re-entry behavior. Closing
stops unread work while retained arrays keep their true owners; final release frees
the lease without retaining the whole completed workspace. Read-only cold admission
finishes before handles are exposed, and later inspection neither constructs
physics nor mutates store contents. Foreign mutable/degraded input establishes
its actual obligations before being re-admitted.

Finish `just engineering-inspection` and existing fresh-store helpers against these
bindings. Run `just py-sync` and `just doctor` before Python behavior tests. Add an
ordinary diagnostic provider and scoped policy through canonical declarations and
actual implementation registration, without core admission or Python policy edits.

**Delete:** parallel inspection lookup/slicing, handwritten competing DTOs and
examples of retired entry points. **Exit:** V01/V14/V15 pass, all four engineering
cases complete cold Python inspection, retained slices outlive closed handles and
the final owner releases its reservation. Store inspection has no write effects.

### PC11 — establish target product and cost evidence

**Owns:** existing engine/conformance/lifecycle tests, `tests/support` engineering
fixtures/expectations, `xtask/src/engineering_inspection.rs`, benchmarks and receipts.

Complete the verification matrix below on the integrated target. The four actual
source-authored heater/mixer × FTPx/FcTP cases must include independent selected
method, equation/root, quantity, contribution/exclusion and provenance assertions,
publication, cold Rust/Python and negative boundary cases. Independent numeric
expectations exercise the sourced physical methods; metadata equality alone does
not qualify physics. Keep external pinned IDAES parity separate and report only
the exercised behavior, with no claim of numerical solving from a compiled graph.

Measure small interactive source changes and larger/skewed inputs, repeated views,
warm/cold resolution and first/cached compilation with workstation-sized budgets.
Record exact thread/partition/batch/budget/spill settings and workload identity.
Measure resolution calls, duplicate lookups, parsed documents, producer invocations,
rebindings, analysis/optimization/physical planning, first/last batch, materialized
bytes, copies/decoding, publication/reopen, peak/residual accounting and spill.

The structural oracles are one necessary local parse/producer invocation, no
compile-on-inspection, no repeated full context reconstruction and no hidden pool
per provider. Investigate and fix material target costs and accidental duplication;
do not assert a universal speedup or create an old-code benchmark baseline. Record
unaccounted process memory honestly. Extend existing recipes where needed instead
of creating a separate performance/acceptance framework.

**Delete:** redundant target work identified by measurements, including unnecessary
materialization, repeated resolution and stale measurement/fixture adapters.

**Exit:** V01–V16 have claim-specific receipts and representative target measurements.
Failures, unsupported cases and incomplete observations remain visible. No focused
suite or benchmark smoke result is treated as the final delivery gate.

### PC12 — delete remainder and close every gate

**Owns:** remaining legacy consumers/configuration/data objects/tests, active
documentation/examples, operation census, terminal receipts and final design review.

Close the deletion ledger below and rerun the callable-operation census from PC00.
For every product entry point, show its native route, actual binding, effective
policy, complete dependency/effect contract, resource owner and observable result.
Inspect direct store/scan/session/algorithm calls to distinguish legitimate native
internals from an independently callable bypass; a text search alone is not proof.
Remove dead adapters, old stores/formats, obsolete tests and unused dependencies.
Generate fresh target artifacts; never stage files merely to hide a hygiene failure.

Run the terminal recipe schedule once on the finished target, fixing failures to
baseline zero. Perform one focused, independent G1–G7 assessment of the actual
implementation and operation/deletion evidence using the design-review workflow.
Each gate needs its own conclusion and executable oracle. This is a separate review
pass by the same Codex stream; delegation is not required. Do not reopen broad
library research absent a concrete gap.

**Exit:** zero callable legacy/parallel product routes, no accepted product exception,
all required terminal checks and seven gates settled for delivered scope. Update
STATUS, this plan and its Outcome with actual receipts and future-wave boundaries.
Only then mark the plan done. Until then, save the exact remaining package/substep
and source/check boundary here; do not create another parallel execution plan.

### Complete operation routing

| Product family | Bound native route | Completion owner |
|---|---|---|
| Query, validation, diagnostics, semantic metadata | Common providers, native plans/functions, residual obligations | PC03–PC05 |
| Sources, external tables, import, schema evolution | Factory/resolver, source providers, explicit parsing/admission/conversion operations | PC03, PC06–PC07, PC09 |
| Namespace/DDL/default/configuration changes | Prepared native logical command, declared private namespace/configuration effects | PC04, PC06 |
| INSERT/DELETE/UPDATE/TRUNCATE/MERGE | Actual TableProvider hooks under one private attempt and publication contract | PC06 |
| Rename and ordered changes | Bound before/after relations, native change plans and complete source correspondence | PC07 |
| P0–P10 and rule closure | Declared native operations, worktable/input providers and typed completed output ports | PC07–PC09 |
| Quantity, MathIR, kernels and existing solver/backend operations | Typed provider inputs and specialized native operator; whole-problem semantics | PC08–PC09 |
| Persistence, Copy and export | DataSink or publication operator, explicit durable/external effects | PC06 |
| Cold open and durable reuse | Explicit resolution/admission operation with shared traversal and exact contexts | PC09 |
| Rust/Python inspection | Same admitted provider owners, metadata and owned streams | PC10 |

### Replacement and deletion ledger

| Existing mechanism | Required replacement or disposition | Package |
|---|---|---|
| Seven-namespace catalog construction; ad hoc role strings | Full scoped inventory with generated names and exact owner binding | PC01–PC02 |
| Root registration no-op | Faithful private native list plus fallible command effects | PC02, PC06 |
| Separate source/role/computation maps | One authoritative binding generation; derived indexes only | PC02 |
| Concrete provider admission roster/settings-query exception | General registered implementation contracts and admitted metadata providers | PC03 |
| Independent capability flags/default/policy maps | Generated descriptive views from actual bindings and canonical policy | PC01, PC03–PC04 |
| Scan-only policy and discarded view/scalar obligations | Pre-lowering binding plus common session/operation enforcement | PC04 |
| Duplicate scan logic, planning-time row evaluation, untrue statistics | Common native scan planning and honest properties | PC05 |
| Mandatory full collection and repeated SessionState/context creation | Owned streams, shared generations and deliberate materialization | PC02, PC05, PC09 |
| Parallel producer dispatch/opaque completion/result adapters | Native operation with private typed ports and one completed result | PC06–PC09 |
| Separate mutation/admission/publication entry points | Explicit native commands and one complete-bundle commit protocol | PC06–PC07 |
| Repeated AST/reference decoding and local producer verification | Shared actual owners and constructive facts | PC07–PC09 |
| Legacy row/truth/graph engines and predecessor data objects | Target-useful algorithms only, bound through native operations | PC07–PC09 |
| Per-stage cold restore/default-policy fallback | Explicit current-format admission with one traversal context | PC09 |
| Direct inspection table lookup and independent slicing | Same provider generation and owned native streams | PC10 |
| Old golden stores, format adapters, old/new tests and stale policy assertions | Fresh target fixtures, independent semantics and current architecture checks | Each replacement; PC12 audit |

Already-deleted `PassOutput`, `PassRecordDraft`, `InputBundle::rows`, `StageContext`,
`TargetContext`, predecessor injection and golden-store generators stay deleted.
This ledger targets duplication and bypasses, not a blanket ban on scalar code,
Cell values, hashing, graph libraries or buffering when a target algorithm needs them.

### Traceability

| Input obligation | Packages |
|---|---|
| Provider review F1 / lookup and identity | PC01–PC02, PC10 |
| F2 / extensible admission | PC03–PC04, PC10 |
| F3 / faithful root mutation | PC02, PC06 |
| F4 / effects and common lifecycle | PC04, PC06–PC09 |
| F5 / views and plans without scans | PC03–PC04 |
| F6 / truthful metadata, facts and capabilities | PC01, PC03, PC05 |
| F7 / coherent resolution | PC02, PC09 |
| F8 / rebinding, streaming and repeated restoration | PC02, PC05, PC09, PC11 |
| F9 / unified inspection | PC10 |
| Provider map §9.A–F / registry, names, absence, mutability, metadata, introspection | PC01–PC04, PC06 |
| Provider map §9.G–H / information schema and capabilities | PC03, PC10 |
| Review §5 / diagnostic, edit, factory/cold, interrupted mutation and adversarial view journeys | PC03–PC10; V01, V04, V07–V09, V14–V15 |
| Review §9 / complete conformance and cost matrix | PC11–PC12; V01–V16 |
| Review §10 / exception and bootstrap boundaries | PC00, PC08, PC12 |
| Plan 05 HP00 / authority and consumed contract gaps | PC00–PC01, PC08–PC09 |
| HP01 / generated Arrow and wire substrate | PC01 |
| HP02 / exact private preparation/results | PC01, PC04, PC06 |
| HP03 / property transfer and one admission program | PC03–PC04, PC08 |
| HP04 / full native execution/ownership | PC04–PC05 |
| HP05 / source/change/P0–P3/evolution | PC07 |
| HP06 / current storage, reuse and terminal records | PC06, PC09 |
| HP07 / finite rule truth, support and closure | PC08 |
| HP08 / sourced reference and physical/MathIR adapters | PC08 |
| HP09 / P4–P6 | PC09 |
| HP10 / P7–P10 | PC09 |
| HP11 / immutable Python and semantic inspection | PC10 |
| HP12 / first-principles engineering and target performance | PC11 |
| HP13 / deletion and terminal closure | PC12 |

Plan 05's LP0–LP7, I1–I9, earlier C0–C7 and Plan 04 domain requirements are carried
through its HP00–HP13 mapping above. Historical migration/predecessor qualification
procedures remain superseded; no functional delivery is silently dropped.

## Verification

### Target acceptance matrix

All target rows are **Proposed** until run on the implemented target. Use existing
crate/test families and meaningful failure cases, not a new certification engine.
The baseline is **0 failures, warnings and unexpected skips**. Tests check the
target contract with independently specified values, states and multiplicities.

| ID | Subject and owner | Required executable oracle |
|---|---|---|
| V01 | Binding identity/discovery; PC02, PC10 | SQL/native/metadata/Rust/Python resolve exact same owners and values for two versions, repeated roles, quoted names, alias changes, absent versus present empty input |
| V02 | Registry/generation/resolver; PC02 | Correct previous owner on replacement, atomic duplicate refusal, removal/cascade, stable captured reads; controlled changing backend, subset/full/negative coverage, errors, cancellation and bounded requests |
| V03 | Facts/metadata/policy; PC01–PC03 | Candidate duplicate/null PK, contextual/composite FK, nested masks, wrong quantity and incomplete coverage give exact findings; metadata/default/capability views match bound implementation; precedence/conflict origins are explainable |
| V04 | Native extension/admission; PC03–PC04 | View, table function, file provider and custom provider bind without a core type branch; wrong owner/forged facts and unknown effects fail; view inlining, folding and zero-row plans preserve obligations |
| V05 | Scan and plan semantics; PC05 | Same immutable data with pushdown versus Unsupported reference agrees in values and multiplicities for nulls/duplicates/projections/filter-only columns/limits; injected over-pruning is caught; truthful statistics and physical properties |
| V06 | Semantic construction; PC03–PC05 | Hand-specified projection/join/union/aggregate/window/unnest/cast cases retain only justified fields/keys/coverage; quantity metadata never performs a conversion; correct ALL-set bag counts and grouping identity |
| V07 | Effect preparation; PC04, PC06 | EXPLAIN/prepare/drop/rejection does not execute rows or mutate; DDL/factory/Copy/session-setting effects occur only in admitted attempts; scalar-only/custom-node hidden effects are detected/refused |
| V08 | Commands and durable lifecycle; PC06 | All five DML hooks on applicable tables, multi-output bundle, count/outcome agreement, late invalid output, cancellation/partial drain, stale base, conflict, lost response and uncertain-commit retry have exact expected visibility |
| V09 | Cold admission and memo; PC09 | Current-format artifacts bind exact parents/settings/implementations; malformed/retired/incomplete data rejects; policy/default/function/source/absent candidate changes invalidate reuse; open cannot repair/publish and lookup cannot compile |
| V10 | Ownership/resources; PC05–PC06 | Pending-stream cancellation wakes; fresh execution resets state; exception/partial drain frees private work; child budget/spill/refusal works; exports retain actual owners until final release |
| V11 | Sources, changes and rules; PC07–PC08 | One changed parse, exact source IDs/scopes, rename/shadowing/stale ordered edits; cyclic finite closure with multiple supports, unknown/conflict/negation and deletion of last external support |
| V12 | Physical/reference/specialized meaning; PC08 | Independent sourced cp/h/s/density, dimensions/affine/reference/basis/domain cases; exact graph roots/argument order/occurrence typing; coupled algorithm input cannot be altered by analytical pruning |
| V13 | Real P4–P10; PC09 | Exact feature/topology/tear/demand/selection/support cases; guards, indexed realization, contributions/exclusions, actual method provision and kernel availability; negative late demand and missing provision errors |
| V14 | Full product and inspection; PC09–PC11 | Four source-authored heater/mixer × FTPx/FcTP cases through real P0–P10, publication, cold Rust/Python; independent physics/graph/provenance, read-only store, retained slices and final zero reservation |
| V15 | Ordinary extension locality; PC10 | Add one diagnostic/native provider and one scoped policy via one authoritative declaration plus actual implementation/tests; no repeated admission/lookup/Python policy edits |
| V16 | Work accounting/performance; PC11 | Recorded target-only small/large/skewed and cold/warm workloads; first/last batch, phase time, calls/parses/rebinds, copies/materialization, publication/reopen, budget/spill/peak/residual measurements; accidental duplication resolved |

Write short premise/conclusion arguments for load-bearing semantic transfer rules.
Tests check their implementation and boundary assumptions; a successful analyzer
or schema comparison is not a formal proof of domain validity. Use metamorphic
checks such as batch/repartition invariance, exact rename identity and current-code
round trips where they falsify meaningful defects. Expected scientific values must
not come solely from the formula implementation under test.

### Command schedule and evidence records

| Point | Command | Evidence limit |
|---|---|---|
| Start PC00 | `just doctor`, required `just bootstrap` path, `just --list` | Environment readiness only; fix pin mismatch without changing declared pins |
| Affected replacement | `just check-library <pkg>` or `just check`; `just test-package <pkg>` with focused target selection | Compile versus behavior claims kept separate; test recipe includes Arrow force-validation |
| Generator changes | `just codegen` or existing bootstrap path, then `just codegen-check` | Actual generated equivalence, not scientific correctness |
| Family/authority changes | `just family-check`, `just adr-lint`, applicable governance checks | One pinned universe and document/structural invariants |
| PC10 Python ready | `just py-sync`, `just doctor`, `just py-test`, `just quality` | Fresh editable target extension and actual Python behavior/import/type checks |
| PC09–PC11 engineering | `just engineering-inspection <fresh-output-directory>`; focused engine/lifecycle recipes | Actual four-case source-to-graph/persist/cold Rust/Python result; reuse the recipe's case selection during development |
| PC11 measurement | Existing benchmark/xtask recipes; add a focused `just` recipe if absent | Record actual workload/settings/artifact identity; smoke is not a performance measurement |
| PC12 complete target | `just ci-pr` | Current recipe includes Rust fmt/check/Clippy/tests/doctests, governance/codegen/family, rustdoc, benchmark smoke, Python quality/tests, ADR and docs |
| PC12 optimized behavior | `just test-release` | Separate optimized force-validation result |
| Existing external parity | `just parity-container` | Pinned IDAES 2.12.0 and exercised scope only; no fabricated numerical solve claim |
| Final documents | `just docs`, `just lint-typos`, scoped `git diff --check` | Document validity only |

Verify recipe definitions at execution time. Serialize native build/test work.
After a terminal command passes, do not repeat its contained suites without a
new change or unresolved concern. Do not add wheel/sdist builds, unused-feature
powersets or legacy comparison campaigns. Existing dependency admission remains
permissive; advisory reports are not new hard gates.

Each receipt records evidence label, command, mode/features, baseline 0,
pass/fail/skip/warning counts, actual source/artifact identity, workload/settings and
what was not exercised. Source inventories and compile checks cannot close runtime
or scientific claims. Historical successful receipts remain historical after changes
to their consumers. If a check is unavailable, report `not_run` and its reason.

### Independent gate closure

| Gate | Required conclusion on implemented target | Main evidence |
|---|---|---|
| G1 | One canonical declaration/binding/policy authority; no competing registry or producer/validator | V01–V04, operation/deletion census |
| G2 | Semantic fields, quantities, identities, multiplicities, source meaning and coupled algorithms survive all boundaries | V03, V06, V11–V14 |
| G3 | Facts arise only from admitted inputs, sound construction or completed obligations; partial/forged results cannot publish | V03–V04, V08–V10 |
| G4 | Effects, external observations and dependencies are explicit; preparation/inspection have their declared behavior | V04, V07, V09, V14 |
| G5 | Atomic visibility/recovery and exact retained generations under failure/cancellation/retry | V02, V08–V10 |
| G6 | Rewrites and reuse retain all semantic premises, including absence, implementation and policy changes | V04, V06, V09, V11, V13 |
| G7 | All claimed capabilities have actual routes and truthful metadata; performance/evidence claims match observations | V01–V16, full terminal receipts and zero bypass census |

### Planning-document evidence

**Proposed:** this task authors the execution sequence and supersession pointers;
no provider architecture is implemented by this plan.

**Tested — documents only, 2026-09-15:** a read-only `.venv/bin/python` check verifies
13 unique ordered packages with responsibility/exit/deletion text, all 10 review
packets, 9 findings, 14 inherited packages, 16 acceptance subjects, 7 gates, 27
existing ADR references and 172 local links across 10 affected files: **0 failures,
baseline 0**. It also checks front matter, balanced fences, whitespace and the
successor-plan anchors/statuses. No repository recipe covers this plan structure;
the check is confined to this documentation task.

`just lint-typos` passes with **0 findings, baseline 0**, after correcting two
spelling findings in the draft. Scoped `git diff --check` passes with **0 findings,
baseline 0**. `just docs` builds with **0 errors and 1 large-search-index warning**,
against baseline 0; that warning remains unresolved. The book includes Plan 06.
These checks establish document consistency, not runtime or scientific acceptance.

## Open items

No compatibility or provider-eligibility decision is pending. PC00 records concrete
authority changes; PC01 settles final internal type names within this design; PC08
closes any missing sourced physical declarations. Choose workload defaults from
PC11 measurements. A current-format version change is allowed when the target
representation requires it; generate fresh development artifacts and refuse retired
versions rather than adding legacy decoders.

If an operation is alleged to be fundamentally incompatible with the framework,
record its required semantics, attempted native extension/operator route, failing
executable example, smallest excluded boundary, accountable owner and revisit
trigger. Difficulty, cost, an existing policy or a non-relational inner loop is
insufficient. No such exception is currently approved or needed by this plan.

This is the requested implementation plan. Further implementation begins with PC00;
ordinary implementation details do not require another planning exercise.

## Outcome (recorded after implementation)

### What was built

Not yet recorded. All PC00–PC12 target work remains Proposed in this plan.

### A mistake made and corrected

Record an actual implementation mistake and its correction at closure.

### Deviations from the plan, deliberate

None recorded. Changes to architecture follow the ADR workflow; routine choices
within these contracts do not add a parallel plan or approval gate.
