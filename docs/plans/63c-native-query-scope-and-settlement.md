---
title: "Plan63c: Native query scope and settlement"
status: proposed
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
| C01 | open | Native context | Accurate readiness and contextual limitation facts |
| C02 | open | Native query | Scoped reference execution and honest graph evidence |
| C03 | open | Native search | Explicit implement-or-retain decision and selected acceptance |
| C04 | open | Native lifetime | Qualified cancellation propagation and actual settlement |

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
