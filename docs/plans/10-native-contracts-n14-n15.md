---
title: Consolidate Delta operations and Python boundaries
status: implemented
date: 2026-09-19
adrs: [ADR-0072, ADR-0073]
evidence: Implemented
---

# N14 and N15 execution

Implements [Plan 10](10-native-contract-consolidation.md), N14/N15 and L15/L17.
N11 was included in the initial request accidentally and is outside this slice.

## Sequence and contracts

1. Resolve execution/storage fields and local predicate templates once. Native Arrow
   casts preserve declared values; Delta CHECK constraints and collection adapters share local
   admission semantics. Cross-relation obligations remain native publication queries.
2. Introduce a catalog-owned operation context with actual session, native resources,
   commit policy and settlement ownership. Configure native builders independently;
   retain the composed Delta planner and require actual SessionState.
3. Share bounded exact-version action reading for receipts and metrics. Distinguish
   rejection, no-op, committed, committed observation failure and unresolved effects.
   Transaction markers supplement exact request reconciliation, never replace it.
4. Derive maintenance from native age floors, protected publications, CDF, attempts and
   leases. Keep automatic log cleanup off ordinary commits; preserve fences and caches.
5. Generate thin Python settings projections from native declarations. Return named
   immutable reports; use checked milliseconds and the Rust identity text codec.
6. Preserve four-state field transfer and native allocation ownership through C-streams.
   Direct errors retain structured diagnostics; stream failures remain inspectable on
   their owning handle because the Arrow ABI itself carries only error text.
7. Delete replaced policies, readers, conversions and APIs; regenerate all projections.

## Verification

**Interface-checked:** DataFusion 55.1.0, Arrow/Parquet 59.3.0, object_store 0.13.2,
Delta revision 58f07cd62bfbce3649a7e1c87c696288068ae184 and the selected overlay;
PyO3 0.29.2, pyo3-arrow 0.19.0 and PyArrow 25.0.1. Local skills and exact source
govern the native libraries; Context7 and selected source govern Python adapters.

Only isolated units, compilation, static checks and pure generation run before N17.
Actual commits/reopen/publication/vacuum/fault campaigns and fresh-store Python
inspection remain N18 obligations. Failure baseline is zero. No performance claim.

## Implementation status

N14/N15 implementation and their scoped deletions are complete. Development verification
is recorded below; final functional qualification remains N18. N16–N18 are still open.

## Implemented architecture

### N14 — Delta contracts and operations

- `delta/operation.rs` owns the actual SessionState, bound declaration, native resources
  and commit policy. It configures write/update/delete/merge/optimize/vacuum builders;
  mutation contexts require the native execution services and preserve caller planners,
  functions and budgets. Exact commands do not rebase and ordinary commits do not clean logs.
- `delta/contract.rs` memoizes session-independent declarations under the registry owner.
  Layout schemas, properties and adapter templates are shared; physical predicates bind
  in the caller's session. The N03 field compiler remains the single semantic lowering.
- `delta/field_check.rs` replaces `nested_check.rs`. SQL lambdas and fields whose storage
  representation differs use a native scalar function that restores the execution type
  through `layout::restore_array` before evaluating the same declared predicate. Arrow's
  native datatype comparison decides whether representation conversion is needed; exact
  field metadata remains enforced by the declaration and scan contracts. Scalar SQL checks
  remain native SQL when no adapter is required.
- `delta/actions.rs` replaces separate receipt and metrics byte readers. It reserves a
  finite decode envelope before fetching a bounded, conditionally identified exact commit,
  checks object identity/range/length, and incrementally decodes the complete action stream.
  Receipts retain one typed summary. CDF reads honor caller cancellation; effect settlement
  uses a distinct observation token so cancellation does not abandon reconciliation.
- `delta/settlement.rs` shares known-version observation failures and preserves original
  operation/recovery causes. Exact domain receipts still distinguish no-op, rejection,
  identity reuse, committed effects and unresolved outcomes. Maintenance failure after a
  durable fence is explicitly distinct from a rollback. Native checksum/cache acceleration
  remains best effort and cannot replay data operations.
- Maintenance admits compact protected ranges under the native pool and expands them once
  for Delta's keep-versions API. One policy combines native data/log age floors with caller
  cutoff, selected snapshots, CDF and unresolved attempts. CDF selects Lite vacuum; active
  attempts retain logs. Native retention enforcement, reader exclusion, fences and cache
  invalidation remain in force. Fractional log ages round conservatively upward.

### N15 — Native Python boundaries

- Native engine/cache input declarations drive constructors, parameter types/defaults and
  getters. Core resolvers own effective settings validation; no PyO3 dependency was added
  to a core crate. Integer extraction refuses booleans, fractions and overflow. Milliseconds
  are unsigned 64-bit values in both directions; native submillisecond truncation is refused.
- Native declarations drive cache/resource report projections. `ResourceReport`,
  `ResourceConsumer` and `TableName` replace positional resource/member results and the old
  `ResourceUsage` API. Nested report sequences are immutable tuples; field names preserve
  `pool_reserved_now`, `pool_peak_bytes` and the distinction from process RSS.
- Generated identity classes delegate textual admission/rendering to `pse-ids`; widths come
  from native constants. Either hexadecimal case is accepted and output is canonical
  lowercase. Content hashes still require the explicit `blake3:` prefix.
- `InspectionError.report` exposes named immutable causes, related findings and N05 native
  execution contexts. Native annotations preserve source spans and grouped notes/help.
  `TableStream.failure` retains structured access to the original terminal native failure,
  independent of the Arrow ABI's textual error slot.
- `inspection/stream.rs` in catalog owns the reusable one-consumption reader state. Python
  supplies the actual executor/reader owner and delegates to pyo3-arrow. Requested schema
  casts refuse; close reaches EOF, cancellation reports failure, and arrays retain N13
  native allocation owners after reader/publication closure. N01's four transfer states
  and complete nested field comparison remain the consumer observation contract.
- All affected active callers, cold-reader tests, Python exports and generator templates
  use the new contracts. There are no compatibility aliases or predecessor property readers.

### Deliberately retained target responsibilities

Publication/member receipt comparison, coherent root selection, reader leases and
execution/storage descriptors remain necessary domain contracts. They use native Delta
transactions and native plans; they are not alternate transaction or query engines.
No Delta overlay change, new workspace crate, integration fixture execution or data
migration was introduced. The direct `chrono` dependency names the already-resolved
0.4.45 native duration type without upgrading a dependency family.

## Development verification receipt

**Tested:** The following commands ran against this working tree on 2026-09-19.
The failure baseline is zero. Rust units use the default nextest profile and explicit
`pse-relations/force-validate`; excluded tests were not executed. Python units use the
editable native extension, Python 3.14.7 and PyArrow 25.0.1. None of these results is a
substitute for A11–A13 functional qualification in N18.

| Command | Result and evidence boundary |
|---|---|
| `just dev-delta-boundaries` | 16 passed, 0 failed, 64 excluded. Isolated contract/layout, action-reader, settlement, retention and C-stream ownership units; no Delta commit or maintenance campaign. |
| `just dev-native-boundary-tools` | 3 passed, 0 failed, 16 excluded. Native export and structured-exception stub declaration units. |
| `just unit-consolidation-governance` | 2 passed, 0 failed. Static development/functional test taxonomy checks. |
| Selected `just py-unit` command below | 67 passed, 0 failed. Native boundary, stub surface, transfer, generated identity and cache settings units. |
| `just lint-native-data` | Passed: workspace all-target compilation and strict Clippy with Arrow force validation. Upstream `proc-macro-error2` 2.0.1 still emits Cargo's future-incompatibility notice. |
| `just typecheck` | 0 diagnostics; tool reports 2 existing suppressed diagnostics. |
| `just lint-imports` | 4 contracts kept, 0 broken. Generated identities have one exact native-gateway exception. |
| `.venv/bin/ruff check python/pse --output-format concise` | Passed for the application package; the repository-wide lint failure is recorded below. |
| `just family-check` | Passed; native family versions and the selected Delta overlay remain unchanged. |
| `just codegen-contracts` | Passed: regenerate native contracts, Python identities and schema documentation. |
| `just py-sync` | Passed: current editable native extension rebuilt and stubs generated from its actual API. |
| `just python-stubs --check` | Passed against the refreshed compiled API. |
| `just py-unit python/pse/tests/test_native_stub_surface.py` | 4 passed, 0 failed after the final extension refresh and stub-generator correction; these are also part of the 67-unit selection above. |
| `just fmt-check` | Passed: workspace Rust and TOML formatting. |
| `just adr-lint` | Passed: 73 ADRs, current index and 31 register rows. ADR-0073 remains proposed. |
| `just docs` | Documentation book built; mdBook reports a large search index warning. |
| `just doctor` | Environment ready after the editable extension refresh. |
| `cargo run -p xtask --no-default-features --locked -- codegen --only python --check` | Passed, 1 schema target. Run directly because the combined recipe stops at the existing Rust tracking failure. |
| `cargo run -p xtask --no-default-features --locked -- codegen --only docs --check` | Passed, 1 schema target. Same combined-recipe limitation. |

The selected Python command was:

```bash
just py-unit \
  python/pse/tests/test_native_boundary_contracts.py \
  python/pse/tests/test_native_stub_surface.py \
  python/pse/tests/test_transfer_contracts.py \
  python/pse/tests/test_generated_contracts.py \
  python/pse/tests/test_native_caches.py::test_cache_settings_refuse_overflow_and_missing_working_headroom
```

Two pre-existing repository-check failures remain visible; no baseline or exclusion was
introduced to suppress them:

- `just lint-py`: 162 findings in the existing DataFusion (42), Delta Lake (98) and
  DataFusion tracing (22) skill scripts. The application package passes independently;
  this slice does not rewrite the user's skill corpus.
- `just codegen-contracts-check`: one generated-file tracking failure for
  `crates/pse-relations/src/generated/runtime/validation_findings.rs`. Rust generated
  content matches, but that pre-existing file is untracked. No staging was performed.

## Outcome

**Implemented:** N14/N15 source changes and their scoped deletions are complete;
N16–N18, final source receipts and functional acceptance remain open. The native Delta
operations share contracts, resources, reads and effect reporting. Python exposes
generated settings, named reports, native identity codecs and structured failure access.

**Mistake corrected:** The initial stub annotation represented `DiagnosticReport` as a
builtin name. The generator now qualifies it through the native module, and its isolated
unit plus Python type checking verify the declaration. Initial integer extraction and
duration conversions were also tightened to refuse booleans, overflow and precision loss.

**Deliberate deviations:** The reusable one-consumption stream state lives in catalog so
its Arrow ownership/error behavior can be tested independently of Python. Generated
identity wrappers may import only the specific native gateway under the revised import
contract. Native domain receipts and lease/fence coordination remain because the required
publication guarantees depend on them. No compatibility API or legacy CHECK reader remains.
