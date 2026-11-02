---
title: "Plan63c: Native query scope and settlement"
status: complete
doc_retention: while-dependent
doc_retention_reason: "Plan63 integrated qualification consumes this package evidence, symbol-search decision and native assurance limits."
date: 2026-11-01
doc_role: plan
doc_owner: docs/plans/63-code-intelligence-cli-robustness.md
doc_topics: [current-work, design, libraries, preparation, validation]
review_sources: [docs/functional_reviews/code-intelligence-cli-robustness-2026-11-01.md]
adrs: [ADR-0233, ADR-0235]
---

# Native query scope and settlement

## Purpose and foundations

**Proposed:** implement O05/O06 and close O07's assurance question, with native O08 cases.
The [coordinator](63-code-intelligence-cli-robustness.md) owns selection and final acceptance.
C01–C04 own native work; B owns shared diagnostic declarations, not native classification policy.

The compatible HIR/IDE family remains 0.0.358, independently prepared from structural syntax0.0.360.
`semantic/native.rs` already forks request snapshots and binds cancellation to the existing
`CpuTask`. `investigation/rust.rs` currently asks for an uncapped symbol population before
file/kind filtering. Reference configuration currently has no explicit `search_scope`.
Graph selection walks the filesystem before native membership/mapping checks.
These are the focused foundations, not a reason to rebuild prepared-world ownership.

## Target and library-backed realization

Readiness identifies the missing prerequisite and the selected actual context: tool available,
service running/drained, configuration/source current, test/macro inputs selected and original
mapping supported. Suggest an existing explicit action only where it can change that limitation.
An absent document binding is not a missing tool; block-local or unmapped evidence is not
necessarily cured by macro preparation.

Separate requested filesystem selection, active native domain, semantic resolution context and
emitted output scope. Preserve declared exclusions, unknowns and per-seed attribution.
An inactive file is not evidence of a missing reference; a limited traversal cannot establish
absence outside its domain. Keep graph support outside the output selection when needed.

Use `FindAllRefsConfig.search_scope` with `SearchScope::files`, `file_range` or intersections
where the selected occurrence predicate lowers exactly. Build that scope from the same
prepared world as the entity; retain definition grouping, READ/WRITE/IMPORT/TEST categories and
plural mappings. Do not reduce the semantic context needed to resolve the item just because
occurrences are restricted. Unsupported selector lowering remains explicit.

Symbol selection retains native exact/prefix/fuzzy/path matching and separate first-N/exists/count
semantics. A small `Analysis::symbol_search` limit cannot be assumed to bound work:
ide0.0.358 constructs `world_symbols` before navigation conversion/taking results.
The present uncapped request avoids dropping eligible candidates after consumer filtering.
Do not introduce that false-negative regression while optimizing the implementation.

Request-local cancellation remains `request_snapshot` → inherited `CancellationGroup` →
`CpuTask::cancel` → actual `settle`. The vendored bridge protects siblings and avoids a shared
database write for local cancellation. Parent-token cancellation alone is insufficient evidence:
`with_cancel_hook` explicitly does not watch parent tokens. Cancellation requested, native result,
physical CPU/macro completion and client-observed terminal delivery retain separate facts.

Critical sources: [IDE reference](../libraries/rust_analyzers/ra_ap_ide.md) §§2–3 and addendum §6;
[HIR reference](../libraries/rust_analyzers/ra_ap_hir.md) §§2–4;
[rust-analyzer reference](../libraries/rust_analyzers/rust-analyzer.md) §§15/20.
Qualify new calls against installed ide0.0.358 and vendored ide_db/Salsa, including
`request_snapshot` and internal symbol-index callbacks. High-level IDE `Analysis` owns cancellable
queries; direct HIR/Semantics has different lifetime/cancellation obligations.
FileId, EditionedFileId, module/expansion and source coordinates are not durable global identities.

Keep existing Buffa/native validation and direct projections. Selected scope/request additions
cross proto only when native execution needs them. B01 owns shared Failure changes; this
companion owns native production and proof of its facts. Proto validation cannot establish
native membership, semantic compatibility or worker drain.

## Packages

### C01 — Accurate readiness and scope interpretation

Trace current preparation/service/source/interpreter checks and graph selection in their owners.
Deliver concrete reasons through B's diagnostic slice. Validate explicit traversal/output roots
before seed lifting using B02 resolution; retain guarded seed hydration independently.
Account for filesystem files absent/inactive in the snapshot and original mapping limits
without treating them as ordinary provider I/O.

Audit the corrected review request's limitations: mapping, block-local nomination, inactive test
modules and edge examination. Make reason/stop/count evidence match actual examination; do not
remove limitations merely to return success. Preserve partial useful nodes and witnesses.

Acceptance: cold source-only reads; unavailable refinement; explicit prepare; drained/stale
session; tests/macros disabled; selected inactive files; interpreter available but binding absent.
Also cover an available tool with a missing or stale document interpreter: native snapshot
freshness cannot establish document-interpreter freshness.
No action silently changes contexts, and no complete no-match is fabricated.

### C02 — Native scoped references and graph assurance

Lower exactly representable occurrence scopes to the native SearchScope mechanism in one world.
Migrate actual reference consumers, wire projections where needed and public schemas with the
producer. Keep graph traversal, output and provider examination independent.

Acceptance: excluded files, included aliases/macros, Unicode token boundaries, multiple native
targets, dependency context, hidden support nodes and any/all-seed attribution. Include successful
positive witnesses as well as restricted/incomplete negative cases. Establish exact expected
scope independently of the lowering under test.

### C03 — Bounded symbol-search decision and selected realization

Mandatory investigation; the operator chose investigate-then-decide on 2026-11-01.
Construct self-contained workloads with many excluded candidates before eligible ones and
inspect exact native execution from matching through navigation/projection. Compare retaining
complete candidate collection with a narrow library extension for selected-scope/accepted-result
stopping. `Query::search` has callback/ControlFlow internally but is not an existing public
streaming contract; do not claim an available API does this already.

Implement the extension only if it preserves matching, context, count/exists/first-N meaning,
original cancellation and independent tests, removes demonstrated repeated/materialized work
for ordinary supported queries, and has a bounded maintained patch/interface. Otherwise retain
the correct population path, document its work behavior and record the specific technical
reason and observable revisit trigger at this package. No arbitrary speedup threshold or new
runtime cost model is required.

Record the decision and evidence here and link it from the coordinator; an implement outcome
requires its producer, consumers, deletion and targeted acceptance before C03 completes.
A retain outcome still requires regression coverage against early-cap false negatives.
Any new vendor patch follows its reconstructable patch-record owner; no family merge or wholesale
upgrade is selected.

### C04 — Cancellation propagation and native contract assurance

Trace explicit Cancel, parent-token, deadline, client disconnect, half-close, dropped consumer
and stop through native worker admission, task hook and actual completion. Half-close permitting
completion must not become cancellation. Correct missing wiring where found; if existing wiring
is sound, record that conclusion with revealing tests rather than add another watcher/registry.

Use deterministic synchronization/barriers where available, not a sleep proving only that time
elapsed. Cover pre-start/in-flight work, cold symbols, references, nested snapshot clones,
sibling completion and a subsequent fresh query. Include delayed macro-worker drain under the
supported explicitly prepared macro profile; never infer it from restricted-world tests.
Consumer delivery failure must not prevent actual native drain or fabricate client-observed
settlement. Refuse unavailable preparation rather than skip a required selected test.

Native O08 controls include malformed/unsupported scope requests, presence/default/oneof/enum
semantics, stale generation/session identity, nested validation detail and invalid outgoing
states. Preserve typed cause/disposition and integer/path fidelity through generated/native/
public lowering. Source-only unusual paths retain their own support boundary; the native VFS
does not acquire unsupported non-UTF-8 capability through a JSON conversion.

## Package state

| Package | Status | Owner | Completion |
|---|---|---|---|
| C01 | complete | Native context | Typed context causes and independent readiness premises qualified; incomplete domains retain witnesses |
| C02 | complete | Native query | Same-world scopes, plural editions/groups, roles and graph traversal-scoped references qualified |
| C03 | complete — retain | Native search | Complete candidate collection retained; adversarial excluded-before-eligible regression passed |
| C04 | complete | Native lifetime | Actual usage-search, gRPC, parent/deadline, managed cancellation/reconnect and macro drain qualified |

## C03 decision: retain the complete native population

**Implemented:** retain the uncapped native candidate request before source/name/kind filtering.
The production policy is named `symbol_candidate_limit`; it is independent of the user's
first-N, exists or count demand. No analyzer family upgrade or vendor extension is selected.

**Interface-checked:** exact 0.0.358 `Analysis::symbol_search` calls
`symbol_index::world_symbols` before navigation conversion and `.take(limit)`.
The vendored `world_symbols` builds the selected indexes, then collects every callback match;
`Query::search` and its `ControlFlow` callback remain crate-private. Merely lowering the IDE
limit therefore removes neither the index/match population nor the risk of dropping the
only eligible file after projection. The existing consumer also filters original paths,
written-name policy and kinds after native navigation.

A narrow ide_db callback extension alone would not alter the published IDE execution path.
Using it for accepted-result stopping would require an additional IDE bridge or bypassing the
current high-level cancellable navigation boundary, and moving the consumer's source selection
and original-source acceptance into that bridge. The self-contained regression demonstrates
the false-negative hazard; it does not demonstrate an ordinary-query navigation/materialization
bottleneck that justifies maintaining this additional interface. Retention preserves native
matching, dependency context, cancellation and all result-demand semantics.

The regression creates 192 excluded declarations followed by one independently selected eligible
declaration, checks that an early 20-result cap loses it, and checks the production candidate
policy preserves it. Revisit when an ordinary supported workload demonstrates repeated or
materialized navigation/projection work dominating the request, or upstream exposes a supported
same-world accepted-result/scope stream. Any extension must then preserve existing exact/prefix/
fuzzy/path matching, plural mappings, count/exists/first-N and cancellation controls.

**Tested (2026-11-01):**
`selected_symbol_population_survives_many_excluded_candidates_before_eligible` passed against
the actual 0.0.358 native engine on Linux with test context and build/proc-macro preparation
disabled. The early-20 control loses the independently selected declaration and the production
uncapped population retains all 193 declarations. This is correctness/work-behavior evidence,
not a latency or memory improvement measurement.

## Native implementation checkpoint

The occurrence contract distinguishes an absent scope from an explicitly empty scope. Whole-file
sets lower to `SearchScope::files` using all module-edition interpretations from the selected
snapshot; one file with one interpretation and range lowers through `file_range`/intersection.
The pinned public scope API has no range-union constructor. Plural ranges therefore restrict
native files first and apply the exact original-byte predicate afterward, reporting that fallback
explicitly. This preserves all interpretations and never narrows semantic resolution context.
An explicitly selected occurrence file with no active native interpretation refuses with typed
`InactiveSource`; it cannot produce a complete-looking empty reference population.
Reference categories and definition grouping remain the pinned native owner's observations.
In particular, the original-definition search records renamed imports but does not establish
complete enumeration of every alias-call spelling; scoped range controls use independently
known original-name occurrences. Shared-module and repeated-macro-argument controls exercise
plural editions and plural native definition groups without merging their identities.

Graph selector roots are resolved before guarded seed hydration. Reference examination uses
the traversal domain, separately from output selection. Inactive and outside-world selected
traversal files retain useful graph witnesses but mark examination incomplete.

Cancellation retains the existing `settle_cpu` parent-token watcher and task cancellation hook.
New deterministic admission observers are test-only. Actual native reference controls cover
production gRPC half-close, explicit Cancel, consumer reset and fresh-query reuse, plus a
native conversation deadline and parent-token propagation. Original cancellation-group,
cold-symbol, graph and delayed macro-worker controls retain their separate evidence boundaries.
An additional test-only thread-local tracing subscriber blocks inside upstream
`FindUsages::search`, reached through the actual `Analysis::find_all_refs` snapshot clones.
It records the original native `Salsa::Cancelled::Local` result after cancellation, separately
from the outer token check, then joins the CPU owner and verifies sibling/fresh-query use.
The admission controls and this in-flight native-result witness establish different boundaries.

## Targeted verification

**Tested (2026-11-01):** the coordinator's native-hir unit invocation
`01a19a6d-70c5-73e4-83da-f0d888fa955d` passed all 49 selected tests with zero failures
against the zero baseline. Original JUnit contains 25 `semantic::native` tests, two actual
native RPC tests and one context-classification test: 28 C controls passed. The other 21
tests exercised shared wire, public failure/redaction, selection/recovery, graph-adapter and
stale-interpreter contracts. The runner selected 49 tests and left 831 outside this focused
selection; its setup testcase is separate from that test count.

The command was `scripts/pse test pse-host --lib --features native-hir` with this selected
filter expression:

```text
test(semantic::native::) | test(semantic::rpc::native_contract_tests) | test(semantic::context::tests::native_context_guard) | test(semantic::wire::tests::) | test(semantic::failure_tests::) | test(cli::investigation::recovery_tests) | test(investigation::providers::selection::tests) | test(investigation::rust::graph_adapter::tests) | test(prepared_xtask::tests::passive_open_authenticates_source_and_evidence_without_a_compiler_or_build)
```

The passed C controls include Unicode aliases/imports, macro-argument mapping, independent
excluded-file and exact-byte-range witnesses, empty scopes and invalid ranges, plural macro definition
groups and one shared module under editions2021/2024; scoped graph roles, hidden support and
seed attribution retain their existing controls. In-flight cancellation reached upstream
`FindUsages::search` on actual IDE snapshot clones and produced an observed original Salsa local
cancellation before CPU settlement; a sibling and a fresh query completed. Actual production
gRPC separately exercised half-close, explicit Cancel, consumer reset and reuse, and the native
conversation deadline and parent-token tests passed. The existing explicitly extended-world
test also passed, including delayed proc-macro completion and original product fencing.

**Focused evidence boundary:** this unit/native contract selection is not the assembled CLI
journey, scope-end static/manual qualification or scientific acceptance. No native source
repair remained from this targeted run. The retained complete symbol population and explicit
plural-range fallback are deliberate choices, not measured performance improvements.

## Verification and handoff

**Proposed:** use existing native unit tests, `composed_investigations` and original
source/association journeys, extending the exact pinned cancellation-group tests as needed.
The existing cold-symbol test proves coarse settlement/sibling survival, not every precise
internal checkpoint; preserve that distinction.
Prepare capabilities explicitly and drain every owned service after functional probes.
Record local Linux features/macros/test-context/target conditions with actual results.
Integrated/static qualification belongs to Q01.

**Interface-checked:** exact source confirms the missing reference scope, symbol materialization
boundary and request-local cancellation seam. It does not establish a reproduced cancellation
defect. No new native preparation, test execution or implementation occurred during authoring.

## Outcome

**Implemented:** C01–C04 are complete, with the C03 retain disposition above. Same-world native
reference scopes precede result caps, preserve plural editions/definition groups and keep graph
traversal distinct from output selection. Inactive/outside-world domains retain their correct
refusal or partial-witness meaning. Existing CPU/macro ownership and cancellation wiring remains
the production mechanism; stronger assurance required no global watcher or vendor patch.

**Tested (2026-11-01):** the focused 49/49 selection above includes 28 C controls. Q01 additionally
qualified its ten selected managed/native/interpreter/graph identities by the initial nine passes
and one repaired authenticated-cancellation rerun. Actual macro stop, sibling survival, fresh
queries and authenticated reconnect were exercised. Assembled native/API/graph source selection
passed its affected reruns. Final native/compiler Clippy and all four host feature combinations
passed with zero warnings/errors. The coordinator Outcome owns the original commands and limits.

Mistakes corrected were admitting Cargo.toml solely because it existed in the VFS, expecting an
alias-call result the pinned analyzer does not enumerate, and placing a socket inside a prepared
fixture. An older managed cancellation fixture also needed the established authentic independent
caller scope; its production detachment guard was preserved. The actual usage-search/Salsa-result
witness complements rather than replaces admission and transport controls.

Deliberate limits remain explicit: plural original ranges use native file restriction plus exact
filtering; complete symbol candidates remain materialized before consumer acceptance; alias-call
enumeration is native-library limited; shared prepared-input changes may classify as SourceChanged.
No performance, universal completeness, scientific or formal ADR acceptance follows. Blueprint
§21.7 owns the enduring contracts and §24.2 the selected qualification basis.
