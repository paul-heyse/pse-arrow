---
title: Native contract consolidation execution inventory
status: in-progress
date: 2026-09-18
adrs: [ADR-0071, ADR-0072, ADR-0073]
phase: 1
evidence: Tested
---

# Plan 10 execution inventory

**Historical inventory:** [Plan 13](13-rust-computation-architecture.md) carries
unresolved acceptance through Plan 11. The
[current checkpoint](13-w19-repair-checkpoint.md) owns resumption; original statuses
and receipts below remain evidence of their recorded source boundaries.

Single execution ledger for [Plan 10](10-native-contract-consolidation.md).
Starting source: `a46f358bdfc2ca27f9f240ab6c045b63141c3ee9`; existing review/plan and
DataFusion skill changes are preserved. Native family and Delta pins/overlay are unchanged. N06 adds independently pinned
`datafusion-tracing` and `instrumented-object-store` 55.0.0.

| Package | Source / consumers / generator | Replacement and oracle mapping | Status |
|---|---|---|---|
| N00 | Guidance, ADRs, justfile, xtask acceptance and benchmarks | Plan 10 receipt/source preflight and focused recipes | complete |
| N01 | P4 scalar helper; Python transfer/inspection; no generator | R8/R10; F11/F16; L01; A04; prior isolated units | complete |
| N02 | Schema resolved graph; relations columnar; catalog output; Rust generator | R1/F2; L02; A01/A03; prior isolated units | complete |
| N03 | Relations validate; capture/checked values; generated builders; Delta admission | R2/R9; F1/F12; L04/L15; A02/A11 | complete |
| N04 | Schema literals/self-description/generators; IDs framing/layout; all Cell callers | R1/R2/R12; F1/F2/F4/F14; L03/L05/L06; A03 | complete |
| N05 | Diagnostics leaf; all domain error projections; registry/Python vocabularies | R12/F13; L16; A14; native source preservation units | complete |
| N06 | Engine extraction, catalog composition, effective settings, production/testkit construction and execution assurance | R11/R12; F8/F15; L07/L18; A07/A14; native units/static checks | complete |
| N07 | Catalog/engine providers, operation families, epoch ownership and metrics | R3/R12; F5/F7; L08; A05/A14 | complete |
| N08 | Native scalar/aggregate/window function wrappers and adapters | R4/R12; F6; L09; A06/A14 | complete |
| N09 | Dependency traversal, configuration classification and reuse keys | R3/R11; F8/F17; L10; A07 | complete |
| N10 | Compiler/template correspondence and source support plans | R6/F9; L11; A08/A15 | complete |
| N11 | Typed graphs, operator bindings and shared exact scalar kernel | R7/R8; F10/F11; L12; A09 | complete |
| N12 | Numerical programs, derivative staging and backend consumers | R7/R8; F11; L13; A09/A15 | complete |
| N13 | Native pools/reservations, buffers, cache and FFI ownership | R5/F3; L14; A10/A15 | complete |
| N14 | Delta operation contexts, mapping/read/commit/retention paths | R2/R9; F12; L15; A02/A11/A12/A15 | complete |
| N15 | Python settings/IDs/durations/reports and transfer consumers; generated stubs | R10/R11; F14/F16; L17; A13 | complete |
| N16 | All fixtures, callers, unused dependencies, generated outputs and docs | R12/F15/F17; L07/L18; A14/A16; grouped campaign-02 repairs implemented and development-checked | complete |
| N17 | Complete package/deletion ledger and exact-source implementation receipt | Grouped repairs closed; machine-checked current-source receipt required for continuation | complete |
| N18 | Final campaign, measurements, independent A01–A16 and G1–G7 decisions | Campaigns 01/02 retained; shared engine and fixture repairs require requalification, followed by remaining gates and review | in-progress |

## Deletion inventory

| ID | Removed authority / consumers | Replacement owner and oracle | Status |
|---|---|---|---|
| L01 | Unchecked P4 division remainder and incomplete Python child dispatch | Checked arithmetic and native transfer traversal; N01 | complete |
| L02 | Compiled-declaration strings, repeated text equality, generated declaration constants | Schema resolved handles and exact one-time admission; N02/N04 | complete |
| L03 | `Cell`, cells/conversion APIs and all active literal/default/self-description uses | Arrow columns and shared lossless literal codec; N04 | complete |
| L04 | Duplicate local field validators, capture PSE walkers and unused bundle validator | Prepared native predicate/obligation compiler; N03/N14 | complete |
| L05 | Mirrored canonical layout and repeated frame/metadata policy code | Native type descriptor and shared IDs kernel; N04 | complete |
| L06 | Repeated generator structural walkers and handwritten JSON escaping | Shared traversal and native serialization; N04 | complete |
| L07 | Generic catalog execution services, old imports/re-exports and duplicate factories | Engine plus explicit catalog composition and shared native fixtures; N06/N16 | complete |
| L08 | Duplicate extension/planner/physical/provider/stream shells | Native operators and explicit shared families; N07 | complete |
| L09 | Incomplete/superseded field-preserving function wrappers | Conformance-driven native adapters; N08 | complete |
| L10 | Scope-blind walkers, unsafe reuse keys and repeated settings classification | Scoped traversal and owned semantic keys; N09 | complete |
| L11 | Decoded relational joins, source-position scans and redundant captures | Native correspondence/support plans; N10 | complete |
| L12 | Binding graph clones, same-purpose traversal copies and duplicate operator dispatch | Shared typed graph views and capability bindings; N11 | complete |
| L13 | Recursive Expr/gradient expansion and duplicate prepared subexpressions | Staged native numerical program; N12 | complete |
| L14 | MemoryReserver/custom reservations/PoolReserver/custom fixed budget and dead forecasts | Native reservations/pools plus narrow lifetime adapters; N13 | complete |
| L15 | Repeated Delta semantic predicates, mapping/builder/retention policies and unbounded reads | Shared contracts, operation context and exact bounded reader; N14 | complete |
| L16 | Duplicate diagnostic vocabularies/classifiers and unused leaf dependencies | Repurposed diagnostic leaf preserving causes; N05 | complete |
| L17 | Repeated Python settings/ID/duration conversion, positional reports and obsolete API aliases | Generated thin boundaries and named reports; N15 | complete |
| L18 | Repeated fixture frameworks, proved unused dependencies/entrypoints and superseded SQL assembly | Testkit, actual production factories and owned rule source; N16 | complete |

## Review and acceptance traceability

The following are scope mappings, not extra completion claims. Package/deletion states
above are the execution authority; independent acceptance remains unassessed until N18.

### Every independent finding

| Review finding | Selected realization | Packages | Acceptance |
|---|---|---|---|
| R1 — exact resolved equality | Sealed admitted handles, complete graph comparison and metadata roles | N02, N04 | A01, A03 |
| R2 — duplicate value validation | Shared native field/obligation compiler; delete Cell validators | N03, N04, N14 | A02, A03, A11 |
| R3 — extension dependencies | Family-specific transfer and scoped semantic evidence | N07, N09 | A05, A07 |
| R4 — lost native adapter capabilities | Provided-method conformance and full field-aware delegation | N08 | A06 |
| R5 — claims/accounting assumptions | Native reservations, allocation-aware transfer/lifetime boundary | N13 | A10 |
| R6 — decoded correspondence | Native compiler/template plans and stable support keys | N10 | A08 |
| R7 — numerical graph expansion | Region-aware shared value/derivative stages | N11, N12 | A09, A15 |
| R8 — arithmetic/edge roles | Guard fix, distinct operator capabilities and typed graph views | N01, N11, N12 | A04, A09 |
| R9 — Delta obligations/retention | Shared lowering, operation context, bounded reads and effective retention | N03, N14 | A11, A12 |
| R10 — Python nested transfer | Native nested enumeration plus exact extension-aware observations | N01, N15 | A04, A13 |
| R11 — semantic reuse keys | Scope-aware owners, syntax/bound-plan separation and explicit settings | N06, N09 | A07, A13 |
| R12 — unnecessary platforms | Two focused crates, diagnostic leaf, shared generators/factories/testkit | N04–N08, N16 | A03, A14, A16 |


### Every disposition of the initial review

| Initial finding | Implemented direction required by this plan | Packages |
|---|---|---|
| F1 — Cell/literals | Native columns and one lossless codec; preserve distinct semantic hash policy | N03, N04 |
| F2 — declaration strings | Exact complete admission then handles; reject fingerprint-only shortcut | N02, N04 |
| F3 — memory | Native reservations plus tested ownership bridge; no naive try-grow/claim transfer | N13 |
| F4 — framing/layout/order | Shared frame kernel and checked native descriptors; retain suitable RowConverter | N04 |
| F5 — extension triples/metrics | Explicit operation families; preserve and complete existing metrics | N07 |
| F6 — function wrappers | Full native hook delegation, rewrapping and field postconditions | N08 |
| F7 — tables | Native memory/stream adapters with truthful properties and epoch owner | N07 |
| F8 — traversal/settings/caches | Scoped evidence, correct child arity, effective settings and bounded reuse | N06, N09 |
| F9 — relational passes | Native correspondence/provenance; specialized indexed work remains explicit | N10 |
| F10 — graphs | Shared typed edge views, cycle witnesses and no whole-graph binding clones | N11 |
| F11 — operators | Separate capabilities, shared exact kernels, guarded native numerical lowering | N01, N11, N12 |
| F12 — Delta | Cohesive module, public builders, shared validation and protected retention | N14 |
| F13 — diagnostics | Leaf class/code declaration, borrowed classification and preserved causes | N05 |
| F14 — renderers | Shared traversal, native serialization and useful generated typed APIs | N04, N15 |
| F15 — fixtures | Dev-only testkit using production factories | N06, N16 |
| F16 — Python | Correct nested transfer, named reports, settings/duration/ID conversion | N01, N15 |
| F17 — reachability/dependencies/SQL | Remove proved dead paths; retain useful kernels; readable SQL and correctly keyed syntax reuse | N09, N16 |


### Acceptance obligations

All rows are **Proposed verification**. Units are implemented with their package and
run before N17; the listed integrated journeys execute only in N18. “Every declared
family” is an inventory generated from actual declarations, not a handwritten subset.

| ID | Claim / required oracle | Owner and final evidence |
|---|---|---|
| A01 | Complete foreign contract admission rejects copied IDs/digests with altered enum/extension/nullability/key/reference/check/policy; equivalent independent owners admit; cyclic references terminate; local handle checks avoid text rebuilding. | N02; adversarial units plus generated/imported adapter boundary cases. |
| A02 | One new nested constraint reaches local admission, native rule output, durable enforcement and publication obligations with consistent decisions/paths. Include hidden children, active alternatives, dictionaries, ranges, quantities, keys and absent references. Arrow safety precedes semantics. | N03/N14; generated field-family matrix and final cross-boundary journey. |
| A03 | No production Cell or declaration-text authority; lossless literals preserve float bits; canonical equivalence and metadata roles are deliberate; self-description and typed/generated fields are exact. No incompatible extra files survive regeneration. | N02/N04; native codec/hash/generation units and final `codegen-check`/consumer checks. |
| A04 | Real P4 arithmetic helper handles integer extrema/zero/nonintegral division without panic or false exactness; actual Python transfer classifier detects map-child changes and preserves its four states. | N01; direct regression cases, broader nested type matrix and N15 consumer integration. |
| A05 | Node families preserve effects/requirements/lease lifetime under rewrite, projection, limit, empty results, reconstruction, reset and cancellation. Epoch replacement respects active readers; shared producer executes once; statistics/metrics stay truthful. | N07; fake-stream/command units and final native pipeline lifecycle cases. |
| A06 | Adapters retain applicable native scalar/array, coercion, field, ordering, simplification, group/window/reversal/statistics/monotonicity hooks and actual identity. | N08; hook conformance matrix, native versus adapted physical plans/results and metrics. |
| A07 | Requirements, absence, membership, multiplicity and source keys remain dependencies. Correlated/recursive/hidden-producer scopes cannot collide. Changed provider/UDF/settings/epoch invalidates; resource-policy changes still re-admit; unknown extensions remain conservative. | N09; adversarial scope units and final clean-versus-reused native computation/Plan 09 cache obligations. |
| A08 | Existing compiler/template outcomes retain typed relations, quantities, chosen bindings, diagnostics and stable positive/negative provenance under reorder/duplicate/missing input. Pure correspondence remains native and does not reconstruct rows only to rejoin them. | N10; operator units, existing complete source-expression/equation compilation cases and final fresh heater/mixer publications. |
| A09 | Graph payload/binding/guard edges retain closure/order/cycle witnesses. Shared value/derivative lowering grows with actual DAG/stages rather than path count; excluded invalid branches stay unevaluated; Jacobians/order agree with an independent oracle. No planning in callbacks. | N11/N12; diamond/guard/payload units, existing numerical/backend tests and linked solver cases. |
| A10 | Fallible native admission and owner lifetime remain correct for shared/sliced/dictionary/foreign buffers, copies, concurrent claims, cancellation and final release. No second reservation universe or claim-induced owner reassignment. | N13; small-pool/owner units and final FFI/cache/storage flows; separately record accounted memory and RSS limits. |
| A11 | Declared execution↔Delta mapping preserves values and full restored field contracts across write/scan/reopen; unsupported lossless mappings fail explicitly. Local/durable obligations agree; builders use the actual session/functions/resources. | N03/N14; mapping units and fresh Delta round trips with exact native/Python inspection. |
| A12 | Faults before/after member commit, publication commit, metrics observation and cancellation yield distinguishable outcomes. Exact retry never duplicates effects; conflicting requests cannot share receipts. Readers/CDF/attempt history survive maintenance and caches observe fences. | N14; policy/state units then fault-injected Delta publication/recovery/vacuum/log-cleanup campaign. |
| A13 | Python settings constructor checks, ID/duration codecs, named resource reports, stubs and nested transfer agree with native declarations; error causes and C-stream ownership survive real publication export/reopen. | N15; units plus final editable-extension unit/component suite and independent Rust/Python fresh-store inspection. |
| A14 | Engine has no catalog/Delta dependency; diagnostics is a leaf; actual algorithm services are storage-independent; testkit is dev-only and fixtures call production construction. Shared causes/codes are preserved. | N05/N06/N16; metadata/import/DAG checks, constructor/classification units and fixture isolation. |
| A15 | Native integration produces compact/bounded preparation and eliminates intended duplicate work; measured claims distinguish planning, execution, IO, memory and code-generation cost. No aggregate speedup claimed from one microcase. | N10–N14/N18; measurement matrix and exact library feature/source provenance. |
| A16 | All review recommendations, L01–L18 deletions, callers/generators/tests/docs and required decisions are closed; no replacement remains an optional runtime path or unfinished TODO. Required final checks have zero failures against zero baseline. | N16–N18; source inventory, final campaign, independent A/G assessment and outcome. |


## Implemented slice

- **N01 / L01:** the real P4 helper uses checked i128 remainder/division; i64::MIN / -1
  remains an exact i128 result. Python traverses all native child fields, dictionary
  values and extension storage; parent reports include descendant degradation. Native
  equality preserves duplicate metadata keys that Python dictionary views would lose;
  ambiguous extension decomposition fails closed.
- **N02 / L02:** `pse-schema::resolved_contract` owns the native graph and sealed handles.
  Registry declarations and generated expectations compare all reachable definitions;
  successful generated proofs remain inside the actual arena. Cycles do not expand
  descriptors; equivalent independent registries bind; copied digests cannot bypass checks.
- Native expected fields are interned by the generator. Small constructor tables build
  them iteratively, preserving the default Rust test-thread stack. Actual Arrow schemas,
  local-value admission and allocation owners remain distinct from semantic equivalence.
- Columnar builders/views, concatenation, foreign rebinding, projection admission and
  catalog output callers use the new contract boundary. Raw schema admission resolves
  against the actual registry. Physical metadata is not silently adapted.
- Deleted the old `compiled_contract` serializer/module/test target, registry string
  cache, batch declaration strings, generated text constants and text comparison callers.
  N04 below replaces the independent value and framing machinery.

## Verification of the prior N01/N02 slice

These receipts predate the N03–N05 changes below; they are not evidence for the current edited source.

All failure baselines below are **zero**. Test execution used the normal pinned toolchain,
existing target cache and default nextest profile; stack limits/timeouts were unchanged.

| Evidence | Command / conditions | Result |
|---|---|---|
| Tested | `just unit-contract-foundations`; `--lib --locked --features pse-relations/force-validate`, only `consolidation_unit::` | 16 passed, 0 failed; 210 other tests filtered out; run ID `814533b8-8a23-4775-901c-6871e161c019` |
| Tested | `just py-unit python/pse/tests/test_transfer_contracts.py`; explicit unit marker, no compiler/publication fixture | 31 passed, 0 failed |
| Interface-checked | `cargo clippy -p pse-schema -p pse-relations -p pse-compiler -p pse-catalog --lib --tests --locked --features pse-relations/force-validate -- -D warnings` | Exit 0; compiles test sources but executes none |
| Implemented | `just codegen-contracts` | Rust/Python/docs generation succeeded; no physical package fixtures |
| Interface-checked | `just fmt-check`; scoped Ruff check/format; `just typecheck` | Passed; Pyrefly 0 diagnostics (2 existing configured suppressions) |
| Interface-checked | `just adr-lint`; `just docs` | Passed; mdBook reports the existing large search-index warning |
| Interface-checked | `just py-sync`; `just doctor` | Final editable extension refreshed; all environment checks pass |
| Implemented | Source deletion search across crates/xtask/Python | No `compiled_contract`, `compiled_declaration`, `COMPILED_DECLARATION` or `for_declaration` references |

The scoped Clippy command is used because the existing `just clippy` recipe covers the
whole workspace and multiple feature modes. `unit-contract-foundations` was added to
unify the selected packages' Cargo feature graph instead of repeatedly compiling different
package combinations. The initial per-package schema/compiler/catalog runs passed;
the final combined result above covers the corrected generated adapters too.

### Corrections established during unit verification

The first unrolled generated initializer caused 4 stack-overflow failures in the generated
adapter tests. Iterative construction corrected all four under the same default stack.
The first Python matrix had one invalid map fixture with duplicate child names; the
classifier correctly refused it. Correcting the key name yielded 28/28 passing cases.
The final metadata probe exposed a separate lossy-dictionary comparison: two native
metadata lists with the same first key/value and different duplicate entries appeared
equal in Python. Native comparison now rejects that mismatch; three added cases bring
the final Python result to 31/31.

The first repository formatting check traversed newly supplied skill evidence snapshots
(297 manifest diagnostics across skill aliases). `.taplo.toml` now extends its existing
captured-evidence exclusion to `skill_improvement/evidence`; no skill content was modified.

## Historical N00–N15 implementation boundary

**Implemented with development-unit evidence:** N00–N15 and their required slice deletions are
complete. N16–N18 remain open. The
[source-qualified completion receipt](../design_review/evidence/native-foundations-implementation-2026-09-18.json)
records N00–N05 inputs. The [N06 receipt](../design_review/evidence/native-engine-implementation-2026-09-18.json)
records the engine/assurance slice. No integrated acceptance or performance claim is made.

### N00 — Inventory, decisions and command ownership

- This ledger maps every N package, R/F finding, L deletion and independent A oracle.
  ADR-0072 records the foundation contracts; ADR-0073 allocates future engine/resource
  decisions. Both remain proposed, with named blueprint follow-ups. The scoped foundation
  review extends design coverage through N05 without accepting unexecuted behavior.
- Acceptance preflight requires every N00–N16 and L01–L18 row individually closed,
  deleted legacy paths absent, and exact tracked/untracked source hashes. Source changes
  invalidate the receipt. Evidence outputs must be ignored rather than enter their own
  input set. Continuation does not certify omitted commands; A01–A16 remain unassessed.
- Pure generation, foundation units and disposable acceptance-tool units have recipes.
  Family checking and Python stub generation use the tool crate without optional compiler/Delta
  fixture dependencies. Benchmark declarations cover cold/reused preparation, evaluation, native value framing,
  shared self-description/handles, diagnostics and generation. They have not executed.
- Fixture-only literal adapters are enabled through development dependencies; ordinary
  production builds do not expose them. `force-validate` retains its Arrow safety meaning.

### N03 — Native validation and obligations

- `ValidationContext` binds the actual immutable registry and SessionState. A private
  registry-owned implementation cache shares the default context; explicit session owners
  prepare independently. Physical expressions are reused without planning each batch.
- One field predicate compiler supplies local occurrence predicates and Delta-compatible
  expression lowering. Arrow structural validation precedes semantic evaluation. Bounded
  native occurrence selection handles null parents, active alternatives, maps/list views,
  dictionaries, run ends and unions. Complete row validity is independent of sample limits.
- The sole declared `runtime.validation_findings` schema retains rule/code, relation ID,
  root row, declared key literals, exact nested pointer, canonical full field JSON and a
  bit-exact observed literal. Cancellation returns an error, never a partial validity claim.
- `pse_local_contract` exposes the prepared implementation inside logical plans. Capture,
  raw admission, generated builder finish and checked-value functions use the shared
  boundary; the checked-value binding follows the actual session owner.
- Relation-ID keyed native inputs feed key/collision/reference/quantity/span/ordinal,
  numerical and artifact obligations. Native UNNEST/range, union extraction and casts
  preserve visible occurrences and actual indices. Missing relations remain distinct
  from present empty relations. Catalog keeps Delta serialization and public builders.
- Deleted the Cell validator, duplicate capture walker and obsolete `validate_bundle`.
  Remaining Delta operation/mapping/retention consolidation is explicitly N14/L15.

### N04 — Native values, framing and projections

- `NativeLiteral` contains one exact field and one Arrow value. JSON is only the scalar
  interchange representation; bulk paths use arrays. Its field-directed codec covers
  native containers/encodings, temporal/decimal/interval types and exact float bits.
  Invalid native type declarations are rejected before Arrow null-array construction.
  Scalar adapters preserve nested fields; defaults bind to actual execution metadata.
- IDs owns one shared native value encoder with distinct lossless-literal and canonical
  NaN policies. Row-token format v2 and registry/relation fingerprint format v2 replace
  their predecessors once. Named-ID derivation and canonical IPC outer framing stay stable.
  Physical, execution and value metadata projections remain distinct; unknown metadata
  survives. Canonical descriptors use native DataType/Field rather than mirrored enums.
- Registry self-description builds native columns from the production declarations.
  Declaration-only bootstrap avoids recursive registry construction and supports empty
  or custom registries. Arrow lexicographic sort/take orders primary keys; cloned batches
  share buffers. No generated-relation dependency is introduced into schema.
- Rust, Python, JSON Schema and Markdown share one native field traversal with target
  policies. JSON quoting uses serde_json. Typed authoring/builders/borrowed views remain.
  Pure regeneration owns output and stale-file pruning.
- Deleted Cell, CellCodec, cells/typed conversion modules, owned dynamic-row decoding,
  duplicated layout/framing/literal walkers, generator conversion blocks and unused
  serde_arrow. All active fixture callers use native views/arrays or a test-only scalar
  literal adapter. DomainKind::Cell and unrelated algorithm Layout types remain useful
  domain concepts, not legacy value representations.

### N05 — Native diagnostics

- Diagnostics is a leaf depending on DataFusion common, miette and thiserror. It owns
  detailed codes/classes, registry/Python vocabulary projections and typed attachments.
- A borrowed traversal preserves Context, External, Shared and Collection structure,
  cause downcasts, order and multiplicity. Owned engine errors share the original native
  tree. Typed domain miette projections use the same code declaration; no string clone
  or message-pattern classifier replaces causes.
- Domain callers, catalog and rules use this boundary. The former catalog classifier
  and unused high-level diagnostic dependencies are deleted. Governance recognizes the
  actual Diagnostic implementation instead of requiring derive syntax alone.

### Remaining plan scope

N09–N16 own dependency/configuration reuse,
compiler correspondence, graph/numerical consolidation, native memory ownership,
Delta operation/storage/retention consolidation, Python boundaries and final caller/fixture
cleanup. L04/L15 remain open to the extent they include that Delta work. N17 cannot seal
until every implementation and deletion row is closed. N18 then runs the integrated,
solver/publication/compiler, full fixture-generation and measurement campaign.

ADRs remain proposed; their named blueprint changes and decision acceptance are not
fabricated by implementation status. Historic physical package fixtures are regenerated
only in the deferred integrated generation campaign. Their source generators and callers
are updated and compile-checked here.

## Verification of the N00–N05 completion slice

Failure baseline is **zero**. The earlier N01/N02 receipts above remain historical.
Current focused units use the unchanged default nextest profile, normal test-thread
stack and existing target cache; no retries or timeout increases.

| Evidence | Command / conditions | Result |
|---|---|---|
| Tested | `just unit-contract-foundations`; six foundation libraries, `--lib --locked --features pse-relations/force-validate`, `consolidation_unit::` only | 84 passed, 0 failed, 213 filtered; run `01adb458-1d85-48b6-917e-58944b82dbe1`; test execution 5.615 s |
| Tested | `just unit-consolidation-tools`; xtask without package-fixture features, force-validate, disposable source-receipt units only | 4 passed, 0 failed, 15 filtered; run `412a2c3e-3c81-4cd3-8a16-84d22a769853` |
| Interface-checked | `cargo clippy --workspace --all-targets --locked --offline --features pse-relations/force-validate -- -D warnings` | Exit 0, including test and benchmark compilation; no test/benchmark execution |
| Implemented | `just codegen-contracts` | All Rust/Python/Markdown/JSON Schema projections regenerated; no physical package fixture journey |
| Tested | `just unit-consolidation-governance`; only two static source-taxonomy checks, force-validate | 2 passed, 0 failed; run `afb32a40-b457-4048-9da3-ddf5d6d30568`; no runtime integration |
| Interface-checked | `just codegen-contracts-check` with a disposable Git index admitting the new generated file | All three targets match; actual user index unchanged |
| Interface-checked | `just family-check`; `just py-sync`; `just doctor` | Exact dependency families agree; editable extension and compiled stubs refreshed; all environment checks pass |
| Interface-checked | `just fmt-check`; `just adr-lint`; `just docs` | Exit 0; 73 ADRs and 31 register rows checked |
| Interface-checked | `just fmt-py-check`; `.venv/bin/ruff check python scripts conftest.py`; `just typecheck` | Exit 0; 150 files formatted, application lint clear, Pyrefly 0 diagnostics (2 existing configured suppressions) |
| Interface-checked; failed | `just lint-py`; repository-wide scope, baseline 0 | 756 diagnostics: 273 under the supplied DataFusion skill, 483 under the supplied Delta skill. Those unrelated skill edits were preserved; application lint above is separately green. |

The ordinary pure-generation check initially refused the new untracked generated
`runtime/validation_findings.rs`. A disposable index admitted only that generated path
for the comparison; the actual user index remained byte-for-byte unchanged. This does
not waive the repository requirement to track generated files when committing.

Cargo reports the existing upstream `proc-macro-error2 2.0.1` future-compatibility notice;
mdBook reports its existing large search index. Neither is a passing assertion about the
upstream issue. Full repository Python lint is not claimed green.
Integration suites, durable Delta writes, full `codegen-check`, numerical solver journeys,
benchmarks and architecture acceptance are **not run**, as required by N17/N18 sequencing.

### Mistakes corrected and deliberate boundaries

Mechanical fixture conversion initially left invalid match patterns and stale imports;
all affected callers now compile, without running their integration journeys.
Native scalar adaptation originally reconstructed nested fields; it now preserves the
actual arrays and explicitly binds default metadata. Malformed datatype admission now
precedes Arrow constructors, preventing a null literal from reaching a panic-prone path.
Markdown was moved onto the same traversal as the other three renderers. Private Delta
validation execution nodes were not imported; shared expressions compose through public
Delta builders. No dependency-family upgrade or vendor-overlay change was needed.

## N06 — Native engine and execution assurance completion

**Implemented:** generic session/provider/preparation/function/planner/policy/inspection
services live in `pse-engine`. `EngineError` is independent of catalog. Native generic
cache/flight/admission/IO services live alongside them; `DeltaCacheService` composes the
shared native service with snapshot/resident adapters and Delta counters. Runtime and
`NativeFixture` use `EngineResources`, with one actual memory pool, reserver, runtime,
store registry and native cache owner. Catalog adds its native Delta planners explicitly.

`SourceWitness` carries an actual opaque catalog-owned selection. Catalog resolves exact
publication members; engine retains actual provider and lease owners without importing
Delta or publication types. The native codec uses version 3 witness descriptors and the
retained providers, with no historical decoder. Domain execution contexts use the generic
session. Durable compiler orchestration remains a legitimate catalog consumer.

Effective settings are resolved against the actual state and projected into semantic,
operational and full inspection views. Unknown settings remain semantic. Functions,
custom planners, native extensions, implementation generation and inherited resource
ceilings survive rebinding. Repeated assembly identifies the actual engine rule by type;
observation wrapping is stripped before semantic rebinding and installed once afterward.

Execution assurance supplies Off/Contract/Diagnostic policies, final native execution
instrumentation, explicit Completed/Failed/Cancelled/Abandoned states and bounded local
capture. Round envelopes include pre-execution refusal and post-stream reset. Both native
async and blocking tasks carry span and dispatch. Direct native physical visitation
exposes properties, downcasts and metrics without building another plan graph. Freshness
is an independent atomic contract state. Delta open/write/publication/maintenance spans
record existing identities and actual native results without creating durable authority.
See [native execution assurance](../dev/native-execution-assurance.md).

**Deleted:** `pse-catalog/src/session`, `pse-catalog/src/provider`, the generic catalog
error module, runtime's reserve module, `tests/support/session_factory.rs`, old candidate
construction helpers, generic catalog re-exports and `SnapshotSession`/`SessionFactory`/
`CatalogError` consumers. Generic-only authoring/rules/backend callers no longer need a
normal catalog dependency. Initial catalog/engine fixtures use production construction;
N16 remains responsible for the complete fixture inventory (L07/L18).

### N06 verification

Failure baseline is **zero**. The default nextest profile, ordinary stack limits and
existing compiler cache were used. No timeouts/retries were increased. Integrated tests
were compiled by Clippy but **not executed**. Earlier receipts are not relabeled as
current N06 evidence.

| Evidence | Command and mode | Result |
|---|---|---|
| Tested | `just dev-native-engine`; selected `--lib`, `--locked`, `pse-relations/force-validate` | 17 passed, 0 failed; 72 other tests filtered out; execution 0.921 s, compilation 23.31 s; run `051fabec-3ef4-4e42-a2b5-983e091c9e51` |
| Tested | `just dev-native-boundaries`; static manifest/error tests, force-validation | 9 passed, 0 failed; execution 0.173 s; run `d718b723-6be5-4af3-8aa4-511469dec9a6` |
| Interface-checked | `cargo clippy --workspace --all-targets --locked --features pse-relations/force-validate -- -D warnings` | Exit 0; all callers compile, no workspace Clippy findings |
| Interface-checked | `just family-check` | Exit 0; 34 DataFusion packages resolve to 55.1.0; Arrow 59.3.0 and object_store 0.13.2 |
| Interface-checked | `just engine-boundary-check` | 0 dependency/deletion violations |
| Interface-checked | `just fmt-check`, `just adr-lint`, targeted Ruff check/format, `git diff --check` | Exit 0 |
| Interface-checked | `just docs` | Exit 0; mdBook warns about the large search index (17,766,310 bytes) |
| Tested, characterization only | Seven isolated library-probe checks, retained exact source/lock/captures in the [probe capsule](../design_review/evidence/native-engine-assurance-probe-2026-09-18.json) | 0 failures; not product integration qualification |

Cargo additionally emits its pre-existing upstream future-compatibility report for
`proc-macro-error2 2.0.1` (`proc_macro` private re-export). This is not a workspace lint
failure and is not concealed by a lint suppression. Python environment/extension
freshness is stale after the lockfile change; editable Python rebuilding and functional
qualification remain unexecuted, under N18.

### Corrections and deliberate limits

- Invocation success is explicit: EOF, failed polls, cancellation and early drop have
  different outcomes. A reusable round's outer result includes reset failure.
- Missing expected telemetry is inconclusive. Native `MetricsSet` remains authoritative
  for metric names absent from a macro's declared fields; previews remain disabled.
- The original family wildcard included an independently released instrumentation crate.
  Metadata now declares that membership exception once; native dependencies remain checked.
- The updated skill added captured build manifests to the formatter's discovery.
  The existing captured-evidence exclusion now covers skill build trees; their bytes were
  preserved, not reformatted. Project manifests remain checked.
- N06 does not claim complete functional assurance or replace semantic/durable oracles.
  At the N06 checkpoint, N07–N16 remained open; the subsequent completion records below supersede that boundary. At that checkpoint N10, N12–N16, N17 and N18 remained open.


## N07–N08 completion — 2026-09-19

**Implemented:** N07 and N08; L08 and L09 are complete. The detailed
[operation/provider family and function-hook matrix](10-native-contracts-n07-n08.md)
records exact ownership, methods, consumption, reset, statistics, cancellation and
field-preservation contracts. At that checkpoint N09–N18 remained open; the
N09/N11 completion below supersedes that boundary.

- Engine `Operation` / `Execution` replace bespoke compiler, rule, solver and Delta
  operation triples. Typed domain bodies retain native execution and domain algorithms.
  Common commands cover Delta write/publish/maintenance/DML, private native DML and DDL.
- Required work is retained before value optimization. Unchanged cache reconstruction
  preserves actual completion identity. Invocation/epoch completion is shared across
  requirements and caches; last-reader abandonment is terminal. Commands/foreign tasks
  retain cooperative settlement and tracing context.
- Immutable providers, epochs and statistics use native memory/stream transports;
  metadata resolution uses the observation family. The generic owner wrapper retains
  resources through reconstruction, fetch and stream lifetime. Native `Statistics::with_fetch`
  corrects pre-fetch native memory estimates for a single partition; multi-partition
  estimates stay inexact when a fetch applies.
- Scalar and aggregate adapters forward all applicable native hooks through focused
  macros, rewrap native configuration/ordering/reversal results, retain actual owners,
  share lossless nested scalar conversion and preserve native scalar fast paths.
  The actual native statistics optimizer remains responsible for substitution; native
  projection metadata restoration retains selected field meaning.
- Deleted compiler/rule/solver planner registration APIs and all active callers;
  deleted separate finite/domain/command/DML/epoch/statistics/resolution shells,
  the candidate module and catalog-owned lease transport. Constructor namespaces,
  typed algorithms, native Delta planner and selected-resident-cache policy remain
  for the semantic reasons recorded in the matrix; no compatibility APIs were added.

### Verification

Baseline: zero failures. No integration or performance command was run.

| Evidence | Command / mode | Result |
|---|---|---|
| Tested | `just dev-native-contracts`; explicit isolated `--lib` selection, default nextest profile, `pse-relations/force-validate` | 34 passed, 0 failed; 100 unrelated tests deselected; native unit execution 0.932 s, cached compilation 3.21 s |
| Interface-checked | `just check-native-contracts`; workspace/all targets, locked, force-validate | Exit 0; no test execution |
| Interface-checked | `just check-native-contracts-solver`; pinned solver interface, Ipopt feature/all targets, locked, force-validate | Exit 0; no solver execution |
| Interface-checked | `just lint-native-contracts`; engine/schema/relations/compiler/rules/catalog/backend/runtime/benches, all targets, locked, force-validate, `-D warnings` | Exit 0; no workspace lint failures |
| Interface-checked | `just fmt-check` | Exit 0 |
| Interface-checked | `just docs`; `just adr-lint` | Exit 0; book built, 73 ADR records and 31 register rows valid |
| Interface-checked | Focused predecessor symbol/path scan and `git diff --check` | No predecessor implementations/callers or whitespace errors in this slice |

Cargo's existing `proc-macro-error2 2.0.1` future-compatibility notice remains an upstream
notice, not a suppressed workspace diagnostic. The earlier N00–N06 receipts retain their
original source conditions; they are not a current N17 seal after these source changes.
Python extension/environment refresh and all functional qualification remain deferred.

### Corrections and limits

The focused tests exposed incorrect shared-cache identity on unchanged reconstruction;
that was fixed, with separate rewritten-value tests retaining invalidation. A native
memory scan's pre-fetch statistics needed the native fetch-statistics helper. New fixture
assumptions about a fresh native struct-field accessor and a shadowed owner were corrected;
no production ownership guarantee was weakened to make those tests pass.

Specialized resident caching remains catalog-owned because generation/version/lease
reacquisition differ from invocation caching. Correlated/worktable producers cannot acquire
invocation-wide completion until their outer bindings are resolved. The N09/N11
record below now covers dependency/correlation and reuse-key consolidation. A05/A06 integrated acceptance
is not claimed; N17/N18 remain open.


## N09/N11 completion — 2026-09-19

**Implemented:** N09 and N11, including L10 and L12. The
[implementation contracts](10-native-contracts-n09-n11.md) describe semantic and
resource boundaries. At that checkpoint N10 and N12–N18 remained open.

- One bounded native traversal retains actual owners, lexical/correlation/worktable
  scopes and effect ancestry. Field admission/derivation, source discovery, freshness,
  codec inventory, cache staging/expansion/rebinding, producer discovery and observation
  share enumeration and reconstruction. Native analyzer/optimizer implementations remain
  authoritative. Structural reuse does not grant effects, completion or read leases.
- Semantic demand includes values, source keys, row presence, multiplicity, requirements,
  absence and order. Native projection analysis crosses hidden cache producers through
  analysis-only native scans; original execution boundaries remain intact. Unknown
  extensions and fixed input algorithms remain conservative. Order-sensitive source
  demands use exact selection/version evidence, never bag equality. Multiple qualified
  roles for the same provider preserve the actual role or all conservative matches.
- Preparation witnesses retain actual registry/provider/schema/function/rule/binding/
  requirement-planner owners plus semantic settings, purpose and policies. Foreign
  witnesses clear structural shortcuts. One versioned classifier keeps unknown settings
  semantic and re-admits operational limits. Delta exact selection, CDF intervals,
  endpoint comparison, leases and maintenance invalidation remain catalog-owned.
- Bounded native SQL syntax caching includes dialect/parser/options and rebinds every
  invocation. Failed/cancelled parsing cannot publish entries; zero disables retention.
  Native cache reports and the generated Python stub include `syntax_bytes`.
- Borrowed `GraphView` edges retain arguments, payloads, guards and ordered kernel
  binding ports. It supplies deterministic postorder, cycle witnesses and guarded regions
  to topo/walk/load/canonicalize/fold/P4/P8/numerical consumers. Both binding-only
  whole-graph clones and P8 child-only reachability are deleted.
- Independent evaluate/fold/lower/derivative/quantity bindings select shared checked
  arithmetic and actual native functions. Pow remains evaluable/non-foldable; folding
  preserves the i64 boundary while P4 uses exact i128 intermediates. Ordered conversion
  and excluded-branch behavior have independent unit oracles. N12 still owns staged
  native numerical/derivative DAG execution.

### Verification

Baseline: zero failures. Integration, compiler/storage/solver journeys and performance
campaigns were not run. No current N17 implementation seal or N18 acceptance is claimed.

| Evidence | Command / mode | Result |
|---|---|---|
| Tested | `just dev-scoped-graphs`; selected isolated `--lib` units, default nextest profile, locked and `pse-relations/force-validate` | 27 passed, 0 failed; 286 unrelated tests deselected; unit execution 0.317 s, cached compilation 22.57 s |
| Interface-checked | `just check-native-contracts`; workspace/all targets, locked, force-validate | Exit 0; compilation only, no test execution |
| Interface-checked | `just lint-scoped-graphs`; slice crates and consumers/all targets, locked, force-validate, `-D warnings` | Exit 0; zero workspace lint failures |
| Interface-checked | `just fmt-check`; Rust and Taplo check modes | Exit 0 |
| Interface-checked | `just python-stubs --check`; actual compiled API | Exit 0; native `syntax_bytes` parameter/property match the generated stub |
| Interface-checked | `just docs`; `just adr-lint` | Exit 0; 73 ADR records and 31 register rows valid |
| Interface-checked, incomplete | `just codegen-contracts-check`; pure generation only | One tracking failure after successful Rust content comparison: existing untracked N03 generated file; Python/docs stages not reached |
| Interface-checked | Scoped predecessor-symbol scan and `git diff --check` | Replaced walkers/opacity helpers absent; zero whitespace errors |

Cargo continues to report the existing upstream `proc-macro-error2 2.0.1`
future-compatibility notice; mdBook reports its existing large search index. Neither
was hidden or turned into a workspace lint allowance.

### Corrections and limits

Native projection rebuilding originally lost qualifiers; synthetic ordinal fields plus
qualified aliases preserve them even with duplicate source field names. Reconstructing
a shared cache initially produced distinct native owners; staging and expansion now
retain the actual computation identity with scope checks for open producers. A traversal
control-table double charge was corrected without raising the existing observation budget.
Final review also corrected same-provider role discovery and propagation of order-sensitive
demand into exact-version catalog evidence.

The Rust contract generator compared its output successfully, then rejected the existing
untracked `crates/pse-relations/src/generated/runtime/validation_findings.rs`. This file
comes from the earlier N03 work. The index is preserved; the check is not reported green.
Its Python/docs subcommands were not reached. Generated Python API stubs were refreshed
through `just py-sync`; no Python functional test ran.

At the N09/N11 checkpoint, native numerical/compiler/resource work and the later Delta,
Python/fixture and terminal work remained in N10 and N12–N18. The following record
supersedes that implementation boundary; prior receipts retain their source conditions.


## N10/N12/N13 native data implementation — 2026-09-19

The [slice execution record](10-native-data-execution.md) is the detailed source and
retained-algorithm inventory. Native resource traits/pools replaced the custom reservation
universe; compiler correspondence carries actual source keys through native captures;
numerical values and derivatives share region-aware physical stages. Source implementations
and predecessor deletions are in place. Development-unit evidence is recorded below.

No N17 receipt was issued and no N18 campaign ran. N14–N18 remain open; prior checkpoint
receipts keep their original source conditions.


### N10/N12/N13 verification receipt

**Tested**, default nextest profile, explicit `pse-relations/force-validate`, baseline zero:

| Command | Result | Boundary |
|---|---|---|
| `just dev-native-data` | 41 passed; zero failed; 195 excluded; 1.776 s test execution | In-memory operator/key/memory/path/numerical units, including tiny native cache spill/readback |
| `just unit-consolidation-governance` | 2 passed; zero failed; no exclusions | Static error taxonomy and library error-boundary checks |

These are isolated development oracles. They do not establish final A08–A10/A15
functional acceptance. The combined recipe includes the rules qualified-capture unit.
N14–N18 remain open. N17 is not sealed and no integration, solver execution, Python
refresh or performance campaign ran. The environment doctor still reports the existing
stale editable-extension/Cargo.lock state; `just py-sync` belongs to the later Python
qualification work rather than this Rust development slice.


**Interface-checked / Tested (static)**, zero-error baseline:

| Command | Result |
|---|---|
| `just check-native-contracts` | Workspace/all-target compilation passed with explicit `force-validate`; no test execution |
| `just check-native-contracts-solver` | Pinned solver-interface compilation passed; no solver execution |
| `just lint-native-data` | Workspace/all-target Clippy passed with `-D warnings` and `force-validate` |
| `just lint-native-data-solver` | Solver-feature/all-target Clippy passed with `-D warnings` and `force-validate` after removing its remaining shared-lease output type |
| `just fmt-check`; `git diff --check` | Passed |
| `just family-check` | Passed; native family pins and Delta source unchanged |
| `just adr-lint`; `just docs` | Passed; proposed ADR retained, no accepted record or blueprint edited |
| `just codegen-contracts` | Rust/Python/docs generated through their generators |
| `just codegen-contracts-check` | **1 failure**: generated Rust bytes match, but existing N03 `crates/pse-relations/src/generated/runtime/validation_findings.rs` is untracked; index preserved |
| `cargo run -p xtask --no-default-features --locked -- codegen --only python --check` | Passed; run directly because the combined recipe stops at the Rust tracking failure |
| `cargo run -p xtask --no-default-features --locked -- codegen --only docs --check` | Passed under the same generation-only boundary |

The Cargo future-incompatibility notice for existing `proc-macro-error2` 2.0.1 remains an
upstream/toolchain notice; strict project Clippy succeeded. The generated-file tracking
failure is not waived and blocks N17 sealing. No git staging, commit or push was performed.


### N14/N15 implementation and deletion receipt — 2026-09-19

**Implemented:** [slice record](10-native-contracts-n14-n15.md) owns detailed source and
verification evidence. N14/L15 replaces duplicate builder policy assembly, receipt/metrics
readers and `nested_check.rs` with bound operations, shared action observations, declared
field restoration and effective protected retention. N03's shared predicates close L04's
remaining Delta boundary. Exact request/transaction/receipt comparison, publication roots,
reader leases and native maintenance remain target responsibilities.

N15/L17 replaces handwritten Python settings/property lists, positional reports,
`ResourceUsage`, independent hexadecimal text admission and duplicated stream state.
Native declarations drive settings and report projections; compiled stubs preserve named
immutable results. The Rust stream owns original diagnostics and native buffers across
C-stream export. Python exposes `InspectionError.report` and `TableStream.failure`.
ADR-0073 records the exact generated-identity import gateway exception.

**Tested**, baseline zero: `just dev-delta-boundaries` ran 16 selected Rust units with
`force-validate`, all passed (64 excluded); selected `just py-unit` ran 67 Python unit
cases, all passed. `just unit-consolidation-governance` passed 2 static taxonomy units.
The slice record includes full commands, static checks and their limits. No functional
acceptance is inferred from these results. The editable Python extension was refreshed.

The combined generated-tree check still fails on the pre-existing untracked
`crates/pse-relations/src/generated/runtime/validation_findings.rs`. Repository Ruff
has 162 findings in the pre-existing DataFusion, Delta and tracing skill scripts; the
application Python package passes its scoped lint. Those gates remain failed against a
zero baseline. User skill changes and the git index were preserved. N16–N18 remain open;
N17 was not sealed and no N18 integration or performance campaign was run.


## N16–N18 implementation in progress — 2026-09-19

**Implemented, qualification incomplete:** the [closure map](10-consolidation-closure.md)
records shared fixtures, intentional native variants, specialized mechanisms, dependency
and SQL decisions, measurement routes and the strengthened acceptance receipts.
[Exact acceptance cases](10-acceptance-cases.toml) bind A01–A16 to actual test
binary/module identities or campaign artifacts. New nested-constraint and post-fence
maintenance cases are authored but have not run. No N17 seal or N18 campaign exists yet.

**Tested:** `just unit-consolidation-tools`, nextest default profile, force-validate,
9 selected disposable tooling units: 9 passed, 0 failed; baseline 0. `just dev-native-engine`,
same profile/feature, isolated native/store/assurance selection: 19 passed, 0 failed;
baseline 0 (93 tests excluded). Tests establish store option/range propagation, one-shot
lost-response semantics, actual pool identity and explicit execution assurance limits.
Further tooling changes require their subsequent unit receipt before N17.

**Interface-checked:** `just check-native-contracts` completed before the final fixture
sweep. Later all-target checks found malformed imports introduced by that sweep and
missing panic documentation on the newly public fault fixture; these are being repaired.
`just family-check` and `just engine-boundary-check` passed, baseline zero; manifests
were subsequently simplified further and their final receipts remain due. No integration,
Delta journey, compiler workflow, solver execution or benchmark has run in this slice.

## N16/N17 implementation closure — 2026-09-19

**Implemented:** this supersedes the implementation boundary of the preceding checkpoints.
N00–N16 and L01–L18 are complete at the source boundary captured by
`build/plan10/implementation-barrier.json`. The seal is machine checked before execution;
a missing or mismatching receipt blocks N18 despite the ledger status. The
[closure map](10-consolidation-closure.md) records fixture ownership, retained specialized
algorithms, dependency reachability, selected bounded choices and measurement routes.
The [case index](10-acceptance-cases.toml) connects all review obligations through their
N/L owners to actual source and binary-qualified functional cases. No missing mechanism
is deferred as qualification, and no acceptance verdict follows from a source search.

**Tested**, default nextest profile, explicit force-validate, zero baseline:

| Command | Passed | Failed | Excluded | Scope |
|---|---:|---:|---:|---|
| `just unit-consolidation-tools` | 13 | 0 | 15 | Disposable receipts, exact binary/parameterization matching, stale/truncated reports, source modes/symlinks, repository case/source index |
| `just dev-native-engine` | 19 | 0 | 93 | Native factory/assurance/store options, consumed ranges, lost response and actual pool ownership |
| `just dev-native-boundaries` | 9 | 0 | 0 | Static crate registration, pins, dependency floors and error taxonomy |

**Interface-checked**, baseline zero: strict all-target `just lint-native-data`,
`just check-native-contracts-solver` (compile only), `just family-check`,
`just engine-boundary-check`, and all three `just codegen-contracts-check` stages passed.
`just adr-lint` passed all 73 records and 31 register rows. `just docs` passed.
The final tooling-unit qualifier fix and fixture planner selections were covered by
strict all-target Clippy after the selected unit runs; they change no asserted behavior.
The upstream proc-macro-error2 future-compatibility notice and mdBook search-index warning
remain visible. No baseline or lint allowance was introduced.

The dedicated blueprint revision 41 reconciles the implemented authority/ownership
contracts; ADR-0073 remains proposed, and accepted decisions remain untouched.
N18 must refresh the stale editable Python extension/environment, execute every required
gate and measurement, then independently assess A01–A16/G1–G7. Integration, compiler,
storage, solver and benchmark execution did not occur before this source closure.

### N18 campaign 01 — disk exhaustion checkpoint

**Tested / Interface-checked, incomplete:**
`just architecture-acceptance build/plan10/acceptance-2026-09-19-01`
retains `checks.json`, exact source hashes, command logs, field-family inventory (443
relations) and generated-source byte inventory (474 files). Against baseline zero,
`fmt-check`, `family-check`, `engine-boundary-check` and full `codegen-check` passed
with unchanged source. Ipopt binding regeneration and three schema targets matched.
The workspace `check` gate exited 101 after four compiler cache-write failures with
`No space left on device`; this is a failed gate, not an accepted test baseline.
No Rust functional suite, solver execution, Python suite, engineering inspection or
benchmark was dispatched. A01–A16/G1–G7 remain unassessed.

The stopped campaign leaves no task-owned test/build process. Completed incremental
compiler sessions older than one hour were inventoried, without deletion, in
`build/plan10/incremental-cleanup-candidates.json` (3,104 sessions; about 107 GiB logical
bytes, before accounting for shared filesystem blocks). Compiled libraries/binaries and
all source/evidence are outside that proposed cleanup. Explicit authorization to clear
these protected build outputs, or another source of free disk space, remains pending.

After sufficient space is available, run `just architecture-seal` again to include this
checkpoint documentation, then continue in a **new** directory with:

```sh
just architecture-acceptance build/plan10/acceptance-2026-09-19-02 \
  --resume-from build/plan10/acceptance-2026-09-19-01 \
  --rerun check \
  --change-reason 'Disk recovery plus checkpoint documentation and CLI argument quoting repair; retained gates have unchanged implementation and generator inputs.'
```

The continuation must execute the failed and remaining gates, preserve all parent
receipts and independently assess A01–A16/G1–G7 before closing this plan. Do not rerun
the already successful broad generators merely because the evidence directory changes.

**Continuation preparation:** external cleanup subsequently restored about 27 GiB of
free space without any deletion by this task. Campaign 02 resumes the failed compile
gate; approval of the bounded incremental-cache cleanup remains pending if further
space is needed. The parent receipt and its four successful checks remain unchanged.

The first continuation command stopped at CLI parsing before creating its output
directory: the recipe interpolated a multiword reason as shell text. The recipe now
uses the existing `[positional-arguments]` / `"$@"` pattern, preserving argument
boundaries. **Interface-checked:** the actual recipe accepts a help request with a
multiword reason containing a literal semicolon (exit 0, no command execution).
Only CLI forwarding and checkpoint prose changed; generator inputs and native
implementation are unchanged from campaign 01.

### N18 campaign 02 — regression repair boundary

User-authorized cache cleanup removed 3,103 specifically inventoried completed incremental
sessions; one was already absent. Compiled artifacts and evidence were retained. The
cleanup receipt is `build/plan10/incremental-cleanup-receipt.json`. The user also freed
external disk space. Workspace `check` and both strict `clippy` modes then passed.

**Tested, failed/incomplete**, baseline zero: the single CI-profile/force-validate Rust
run selected 1,075 tests across 156 binaries. It ran 1,037: **944 passed, 92 failed
(including three cases interrupted by the deliberate stop), one timed out**; 38 were
not run. The run was interrupted after the shared failures and a 360-second timeout
were established, while two heater cases exceeded 500 seconds. No timeout or retry
setting changed. The failed gate and native JUnit report remain in campaign 02.

A failing property test wrote
`tests/conformance/tests/canonical_properties.proptest-regressions`; this is the only
source change detected during execution. Preserve that reproducer. Accordingly the
runner records source drift and does not represent the test gate as unchanged-source
acceptance. `build/plan10/failing-tests.json` contains the 93 failed/error entries;
`campaign02-source-changes.json` records the source difference.

N17 is reopened for grouped fixes: native expression/lambda/qualified-field preparation,
private DML target ownership, Delta obligation preparation and fixture planner wiring,
validation versus deliberately invalid test inputs, native allocation/error contracts,
canonical vectors/generator assertions, recursive preparation/stack safety and the
engineering timeout. Resolve the complete failure inventory with focused development
checks before further integration. Then reseal and continue the required gates using
linked receipts, invalidating all gates affected by the repairs. No final A/G verdict,
benchmark result, solver/Python qualification or plan Outcome is established yet.

### Grouped repair checkpoint after campaign 02

**Implemented, integrated acceptance pending:** direct predicate preparation now uses
the native simplifier (including `coalesce`), lambda variables resolve before field
admission, compiler joins retain qualified fields, and private DML targets retain
their actual bound provider identity. Delta fixture sessions install the real Delta
planners. CHECK UDF results retain batch cardinality for empty inputs. Diagnostics
preserve the concrete domain cause through the standard Rust error chain.

Contract protection now walks actual native plan owners iteratively, within separate
subquery/cache scopes, with a finite reservation and cancellation. The 256-projection
probe previously overflowed; both the protector probe and the rule witness probe now
pass under the unchanged default stack. This is not yet proof that the seven original
stack failures or engineering timeouts are all resolved.

**Pin-specific correctness decision:** `EngineFactory` disables
`datafusion.optimizer.enable_leaf_expression_pushdown` using the native typed option.
In DataFusion 55.1.0, `split_and_push_projection` compares only output-name sets when
deciding whether to restore a projection. The isolated source-span obligation over an
in-memory decoded view reproduces lost Delta conversions and a schema invariant failure.
The same probe passes with this native option disabled. Other native optimizer rules,
caller functions and provider contracts remain in use. Revisit when the pinned rule
preserves expressions and complete fields; do not restore it on a name-only comparison.

Physical diagnostic observations now include independently planned cache producers,
while optimizer children retain their intended boundary. The bounded native traversal
does not execute producers. Negative validation fixtures explicitly encode untrusted
Arrow through the shared test-only codec, and local check oracles examine native findings.
Per-invariant conformance fixtures exercise their declared native SQL independently of
earlier local value admission. The complete product invariant journey remains required
in N18. Obsolete fixture directories are removed by `just conformance-fixtures`.

**Tested**, default nextest profile, explicit `pse-relations/force-validate`, zero failure
baseline; these are isolated development checks, not a repeated integration campaign:

| Command/filter | Result |
|---|---|
| `just unit-package pse-relations 'test(validate::prepared::consolidation_unit::) or test(validate::predicates::consolidation_unit::)'` | 11 passed, 0 failed |
| `just unit-package pse-engine 'test(unresolved_higher_order_variables) or test(rust_insert_uses_native_defaults)'` | 2 passed, 0 failed |
| `just unit-package pse-catalog 'test(delta::field_check::delta_boundary_unit::)'` | 2 passed, 0 failed |
| `just unit-package pse-engine 'test(deep_native_contract_protection) or test(protecting_pure_requirements) or test(explain_does_not_execute_commands)'` | 3 passed, 0 failed |
| `just unit-package pse-diagnostics 'test(shared_nested_causes)'` | 1 passed, 0 failed |
| `just unit-package pse-numerics 'test(constant_power_exponents)'` | 1 passed, 0 failed |
| `just unit-package pse-rules 'test(deep_native_projection_graph)'` | 1 passed, 0 failed |
| `just unit-package pse-catalog 'test(source_span_obligations_preserve) or test(nested_leaf_extraction_preserves)'` | 2 passed, 0 failed |
| `just unit-package pse-ids 'test(v2_row_token_matches)'` | 1 passed, 0 failed; independent v2 wire framing supports the updated public frozen vector |
| `just unit-package pse-engine 'test(diagnostic_physical_graph) or test(session::admission::tests::) or test(session::config::tests::)'` | 14 passed, 0 failed |

The remaining diagnostic source-chain probe passed (1/1), both Delta admission fixture
probes passed (2/2), and compiler field qualification passed six expansion/index cases.
The same seven-case compiler run exposed one further seed qualifier defect; after its
repair, the seed-only rerun passed (1/1). This is a six-plus-one continuation, not a
claim that the original seven-case command passed. The exact commands and logs are
preserved in `build/plan10/development/` and the development receipt.

**Interface-checked**, zero failures against zero baseline: `just check` (workspace,
all targets), `just clippy` (workspace/all targets in both default and no-default-feature
modes, `-D warnings`), `just fmt-check`, `just family-check`,
`just engine-boundary-check` and all three `just codegen-contracts-check` stages passed.
`just conformance-fixtures` regenerated the invariant cases successfully, including the
new validation-findings primary-key fixture. It generates inputs without executing
product admission, compiler, storage or solver journeys. The existing upstream
proc-macro-error2 future-compatibility notice remains visible.

**Implemented:** N16/N17 are closed again. Every campaign-02 failure is assigned to its
repair or requalification obligation in `build/plan10/grouped-repair-inventory.json`.
The independent engineering assertions and unchanged timeouts remain required; their
previous timeout/interruption is not claimed fixed by the small traversal units.
No additional integration, publication, solver or benchmark campaign ran during the
grouped repair. Campaign 03 must requalify the shared engine changes across the Rust
suite, then continue the unexecuted gates. It must also rerun the affected compile,
lint, format, family and boundary gates. Prior full generated-output equality may
remain linked because no generator or generated contract output changed in this repair.
