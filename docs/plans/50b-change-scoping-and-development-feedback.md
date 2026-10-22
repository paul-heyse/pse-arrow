---
title: "Plan 50b: Change scoping and development feedback"
status: draft
date: 2026-10-22
adrs: []
review_sources: [docs/functional_reviews/codex-effectiveness-and-diagnostics-2026-10-22.md]
doc_role: plan
doc_owner: docs/plans/50-codex-effectiveness-and-diagnostics.md
doc_topics: [environment, diagnostics, documentation, validation, performance]
---

# Plan 50b: Change scoping and development feedback

## Purpose and current foundation

Deliver O03–O05: identify real consumers and useful tests, retrieve the right command/context,
and explain what builds are doing. The [coordinator](50-codex-effectiveness-and-diagnostics.md)
owns progress and shared scope. **Interface-checked:** source `bb61ae3f5`; **Proposed:** target and
verification. This is a local navigation/feedback capability, not automatic architecture approval
or proof that every changed behavior has a test.

`affected` already maps staged/unstaged/deleted/untracked inputs to Rust packages and reverse
dependencies, widening root build changes and naming uncovered categories. Rust-analyzer has
its own target subdirectory and selected validation feature. Pyrefly 1.3.1 already supports
`lsp`, references and other semantic services, with background indexing/coverage constraints.
The compiled product catalogue and generator declarations provide cross-boundary leads that
neither language server can infer alone.

`just --list/--usage/--show` and JSON dumps own recipe grammar. Document lifecycle queries own
derived native plan scope. Assessment and frozen-review readers already demonstrate short
summaries and selective detail. Build/cache/admission owners provide facts but no joined
command-stage explanation. Preserve their strengths; avoid another catalogue or job ledger.

## Target design

Use native semantic queries for references/implementations/call hierarchy and text search for
dynamic configuration, authored vocabulary and generator declarations. Compose explicit
cross-boundary leads from the existing catalogue and source owners: declaration→generator→Rust
consumer→PyO3→generated stub→Python facade→tests. Each lead states its source and coverage;
an unresolved dynamic consumer remains unresolved. Code graphs, scientific graphs and document
scope remain distinct.

Extend `affected` with a structured preview projection that shares its existing selection logic
with text output. Preserve the exact nextest expression and force-validation route. Report
Python, native, generator and external-process omissions; do not silently run all categories or
invent a second test-selection manifest. Add links/commands for those actual owners.

Command/task context uses a read-only projection over existing recipe declarations, native
scope tables, architectural owner routes and selected Git changes. It may recommend existing
commands but never executes them as a side effect of inspection. Derive grammar/defaults;
author descriptions only for effects and workflow meaning. Query by task/owner/changed package
and return a small relevant selection with continuation/raw references. Keep ordinary CLI flags
and direct tools usable.

Build progress combines actual Cargo JSON/compiler records, linker/build-script processes,
cache observations and C02's wait evidence. A command can be compiling one large unit despite
sixteen jobs being selected. Cargo lock-owner attribution remains unknown without an authentic
association; shared cache counters remain shared. Do not infer build freshness or scientific
validity from a stale summary or configuration text.

## Packages

<a id="B01"></a>

### B01 — Semantic navigation and bounded impact leads

Use A02's session attachment for real Rust reference/implementation/call hierarchy queries.
Test feature-gated consumers and macro-generated declarations with scope visible. Resolve I01
by checking a maintained adapter for pinned Pyrefly's stdio LSP; prefer that server before adding
another Python type universe. If no existing bridge fits, select a maintained LSP client library
and expose only the needed read queries; do not hand-write the protocol or build a semantic index
as a prerequisite. If useful Python navigation cannot be supplied, keep that obligation visibly
open or explicitly narrow it with the operator rather than mark it delivered by Rust navigation.

The actual consumer journey traverses a Python facade, generated `_native.pyi`, PyO3 method,
Rust implementation and their tests. The cross-language step comes from the catalogue/source
owner, not a fictional LSP jump into native code. Preserve background-index readiness, file limits
and unsupported/dynamic reference coverage. Add structured `affected` output using current
declarations and retain empty-selection refusal in the execution route.

<a id="B02"></a>

### B02 — Task-shaped command and context retrieval

Expose a fixed discovery route over recipe metadata and current native document scope, with
task/owner/file filters and a JSON projection. Choose the existing command/discovery owner for
its CLI; no new operational daemon or editable registry. First consumers are package checking,
run inspection, query inspection and resuming one named plan package.

Summaries show assumptions, effects, selected mode, concrete command and output handle. Source
and report readers retrieve selected fields/pages/tails rather than whole nested manifests.
Do not print raw process environments or credentials. Context identifies proposed, implemented
and evidenced states and links the native owner; it does not rewrite plan state or infer
completion from source resemblance. Include a bounded retrospective extraction route only as
an explicitly invoked tool, with repository/time scope and inherited-record deduplication.

Remove superseded repetitive instructions after actual consumers move, keeping canonical
governance concise and detailed mechanics at their owner. Do not replace Markdown authoring or
resolve Plan32's reopened representation question here.

<a id="B03"></a>

### B03 — Explain build stages and contention

Have the existing build invocation owner publish transient stage/progress facts for its actual
Cargo child and expose them through activity/run inspection. Use Cargo message records and
actual process identity, not process command text as a control authority. Differentiate compiler,
linker, build script, cache behavior, Cargo lock wait and host admission; retain unknown stages.

Use existing compiler timing/cache recipes for deeper inspection. Record cache observation
scope and actual bypass reasons where the wrapper owns them. Avoid a universal build-key proof
system or forced rebuild to establish the report. Stage age is elapsed observation, not an ETA.

Exercise a cold fixture build, a small rebuild and controlled concurrent artifact producers;
verify reported stages/unknown lock ownership. No machine performance threshold is required.
Preserve sixteen-job/unbounded defaults and the known need to run incompatible Clippy feature
graphs sequentially. Independent check work stays parallel; isolated worktrees are available
for genuinely independent artifact-producing work, never a shared target directory.

<a id="B04"></a>

### B04 — Close the optional search/knowledge decision

Resolve I09 with a concrete question such as finding the current owner and consumers of a
renamed semantic operation across code, generated declarations and active documents. Compare
native symbol/text/document retrieval against an available maintained semantic search mechanism.
An adopted index must be derived, scope/revision-aware and rebuildable, and answer that consumer
without becoming a second authority. Otherwise record no-adoption and retain the direct route.

Similarly select only source-backed agent knowledge that helps the actual pinned integrations;
do not install every ecosystem skill or ingest unrelated repositories/transcripts. This package
ends with a decision and any positively selected bounded integration, not a generic future search
platform or periodic review duty.

## Acceptance and integration

**Proposed:** independent expectations for structured selection/exclusions; recipe argument and
effect descriptions; source/stub/native navigation; partial-index reports; selective output and
actual build stage controls. Pure preview tests need no solver/store. A02 supplies client access;
C01/C02 supplies run/wait identity. B02 can ship its recipe/document subset before those consumers.

Broader developer-effort savings are unmeasured. G02 exercises actual fresh-session/Rust/Python
changes; it does not promote package reach to exhaustive behavioral coverage. Source routes:
`scripts/affected.py`, `scripts/build_environment.py`, `scripts/compiler_cache.py`,
`scripts/document_lifecycle.py`, justfile metadata, the existing catalogue and the
[environment guide](../dev/agent-environment.md). Primary mechanisms:
[rust-analyzer configuration](https://rust-analyzer.github.io/book/configuration.html),
[LSP extensions](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/lsp-extensions.md),
[Pyrefly IDE services](https://pyrefly.org/en/docs/IDE).
