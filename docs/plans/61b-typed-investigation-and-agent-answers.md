---
title: "Plan61b: Typed investigation and agent answers"
status: complete
doc_retention: while-dependent
date: 2026-10-31
doc_role: plan
doc_owner: docs/plans/61-code-intelligence-foundations.md
doc_topics: [current-work, design, diagnostics, libraries]
review_sources: [docs/functional_reviews/code-intelligence-protobuf-foundations-2026-10-31.md]
adrs: []
---

# Typed investigation and agent answers

## Target and foundation assessment

**Proposed:** typed consumed observations and compact usable answers for O01/O02/O12/O13,
with supporting O06/O07. [Plan61](61-code-intelligence-foundations.md) owns dispositions.
Current `investigation/{model,composition,rust}.rs`, `semantic.rs` and CLI presentation already
own demand, truth, source guards and finality. Replace interpreted category conventions where
consumers make decisions, not every legitimate opaque provider document.

`predicate_truth` currently looks for `predicate_evaluation` then a boolean/holds JSON field.
Symbol/importability/location consumers likewise interpret provider-shaped fields. Use generated
typed facts and explicit unknown/refusal instead. Original LSP/rustdoc/compiler evidence remains
qualified in its format, provider and source world. Descriptor full names cannot establish
native identity. HIR objects/binders stay owner-local; original/adjusted types, macro fallback
and exact source, references and actual selected calls retain their different meanings.

Use [HIR](../libraries/rust_analyzers/ra_ap_hir.md), [IDE](../libraries/rust_analyzers/ra_ap_ide.md),
[rustdoc-types](../libraries/rust_analyzers/rustdoc-types.md) and the relevant syntax/edit
references for these mappings. Preserve actual preparation/capability limits; no new compiler
driver or blanket multi-world analysis is implied.

## Packages

<a id="b01"></a>
### B01 — Local typed facts and the shared projection slice

Consumes working A02. Deliver reusable fact constructors and direct typed/reflected access with
real in-process source/API consumers. Start with source declaration/location and qualified API
evidence needed by existing inspection and D01. Do not require C02 to make those local paths
useful. Native helper engine outputs and their wire callers migrate together in C02, not through
a temporary JSON adapter here.

Use generated owned messages where they fit. Eager view/OwnedView and direct VTable reflection
serve selective consumers without a DynamicMessage transcode; convert once when an actual task
lifetime requires owned engine input. Avoid holding large backing buffers just to retain tiny
facts. Keep semantic construction, contextual qualification and original opaque evidence explicit.

Acceptance: an actual source/API question consumes typed facts; a same-spelled entity, stale
source, absent artifact and unsupported fact remain distinguishable. Inspect selected fields
through the shared reflective consumer and retain exact source/native path guards. Remove
duplicated local JSON interpretation and migrated conversions, preserving original artifacts.

<a id="b02"></a>
### B02 — Composed queries, compact answers and batch meaning

Consumes B01; complete native behavior consumes C02. Source-only work can migrate earlier.
Keep source selection, predicate scope, answer demand and rendered allowance independent.
Reuse existing All/Any/Not/refinement and query validation; do not introduce a process-specific
DSL. One Start may carry set/batch intent over one selected question/world. Native preparation
and request-local scans/proof reuse should not repeat per returned fact.

Return typed witnesses, shared context, explicit unknowns and useful guarded expansion.
Existence/limited answers can stop early; complete absence/exact counts require sufficient
examination. Stable cursors retain their original premises. Use typed presentation views with
Serde/Schemars output direction; full ProtoJSON is an optional qualified interchange view,
not the human/agent UX. Do not infer generic applicability from a shape match.

Acceptance exercises actual human/JSON/NDJSON output, saved/flag requests, source-only reads,
native refinement and one expansion. Include truncated demand, unknown under negation, stale
continuation, raw path bytes and missing terminal delivery. Small rendering cannot falsify
coverage or force an unrelated complete scan. Migrate current query/result formats and consumers
deliberately; remove old decoders/oversized extraction paths when replaced.

<a id="b03"></a>
### B03 — Diagnostics, typed clients and workflow adoption

Consumes B02 and the generated C02 client. Expose schema/constraint/context/provider/delivery
failure phases through the existing diagnostic owner with original causes and useful corrective
locations. Distinguish invalid request from validator defect; structured error transport must
not erase original native refusal or CPU completion facts.

Deliver a minimal reusable typed caller for current CLI/helper needs and batch intent, without
a second orchestration layer. Update actual CLI help, schemas, saved examples and targeted
`pse-cli`/process references with decision-oriented uses. Publicly exposed capabilities should
be discoverable through the existing parser and schema routes, not a new hand-maintained catalog.
Use RPC-aware tracing for selected operation outcomes; no full-payload logging or collector
dependency for passive source work. Prefer installed external clients for occasional service
interrogation over an embedded universal dynamic-invocation subsystem.

Acceptance follows an original failure to its affected source/context, a generated typed client
query through settlement, and optional guided expansion without coordinate copying. Broken
saved-query forms migrate in the same cutover; replaced advice and mechanism-only tests retire.
Remote/browser clients and durable sessions remain conditional, not requirements for B03.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| B01 | complete | Local evidence/provider owner | Actual source/API consumers use generated facts and borrowed direct VTable projection; source/API identity tests pass |
| B02 | complete | Query/answer owner | Typed source/native composition, shared contexts, use-site-qualified proof/client reuse and guarded expansion exercised through public journeys |
| B03 | complete | CLI/client adoption owner | Typed diagnostics, original settlement, reusable generated caller and actual help/schema/guidance integrated and exercised |

## Verification

**Tested — 2026-11-01, zero failure baseline:**

```text
scripts/pse test pse-host --lib -E 'test(plan61_) | test(semantic::wire::) | test(code_intelligence_contracts::) | test(semantic::protocol_facts::) | test(semantic::failure_tests::) | test(investigation::composition::) | test(investigation::config::) | test(investigation::stream::) | test(typed_api_projection)'
```

All 61 selected default-feature tests passed. These include direct source/API projection,
all eighteen operation mappings, malformed fields, raw path bytes, checked previews,
unknown predicates under negation, count/existence beyond twenty, saved request/configuration
precedence, typed failure phases, original cancellation outcomes and fourteen controlled
production-router/worker RPC scenarios. `scripts/pse test pse-host --test investigations`
passed all fifteen public source journeys, including human/JSON/NDJSON output, raw source paths,
stable/stale continuations, bounded rendering and missing finality.

**Tested:** top-level `cargo t -p pse-host --test composed_investigations` passed all four
actual prepared-helper/API journeys. They exercise native namesakes, refinement, relationships,
inspection, source-bound proposals, test selection, importability, source equivalence and
distinct Cargo worlds. The selected live-LSP and managed cancellation/reconnect journeys in
[61c's verification](61c-local-rpc-and-native-completion.md#verification) also pass. Independent
session fixtures require that documented top-level route; attempts inside a managed test or
environment workload correctly refused independent detachment and receive no acceptance credit.

**Interface-checked:** default-feature `scripts/pse-env -- cargo check -p pse-host --all-targets`
and `scripts/pse check pse-host --features native-hir --all-targets` passed against zero
errors/warnings. Later helper-version changes were compiled and exercised by actual preparation
and its focused metadata test. Q01 retains whole-series hygiene and assembled qualification.

**Tested:** full `just codegen` published all ten schema targets without further tracked output
changes; `target/debug/xtask codegen --only code-intelligence --check` passed the affected
contract comparison. `just ready` passed skill synchronization and doctor after dependency
wiring changed. All checks retain a zero failure baseline; no whole-series hygiene claim follows.
`just turn-end` passed. `target/debug/xtask lifecycle validate` passed after moving the existing
A/C/D package tables directly under their declared headings so the current dependency/status
bindings resolve; D package scope and state were preserved.

## Outcome

**Implemented and Tested:** B01–B03 are complete within the selected source/native/API scope.
Consumed facts no longer depend on category-sensitive generic JSON interpretation. Original
provider evidence remains separately inspectable. Source/API consumers use generated facts and
borrowed direct VTable projection; ordinary callers decode native observations without HIR
execution dependencies. Existing grouped predicates supply set intent without a second DSL or
orchestrator. Request-local contexts, proofs and authenticated connections are reused while
retaining original use-site, source, artifact and world guards.

The independent implementation review found and corrected the HIR-only DTO gate, call-selection
range regression, overly broad proof reuse, path presentation failure, lost composed settlement,
discarded validation locations, symbol-kind normalization and incomplete terminal validation.
Actual helper preparation also exposed its unmigrated version-one verifier; the generated-service
and shared-version correction is exercised. Two mistaken test selectors were corrected without
weakening the tested behavior. Earlier failing attempts are not passing evidence.

**Limit:** the pinned native analyzer VFS accepts UTF-8 paths; unsupported native raw-path queries
refuse explicitly. Source-only raw-path access and transported/native presentation bytes remain
preserved. No measured context/latency reduction, arbitrary trait applicability, remote deployment
or full-product qualification is claimed. D01–D03 and Q01 remain separate work owners;
this completed companion is retained while those depend on it.

**Proposed:** targeted model/provider/render/client tests accompany each mechanism. Apply actual
force-validation requirements to affected Arrow closures. Q01 owns integrated/static checks;
context/token/latency reductions remain hypotheses until independently measured with comparable
questions, evidence and necessary expansion. No numerical target is required for completion.
