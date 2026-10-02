---
title: "25h: Authoring and admission ownership"
status: done
date: 2026-09-30
adrs: [ADR-0150]
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s06]
---

# 25h: Authoring and admission ownership

## Context and target

F19/F20/F31 and FU08 concern late reparsing, inactive authorities and context-dependent reuse.
R9 requires admitted occurrences rather than syntax alone; R10 requires complete interpretation
context before wider caching. A1 owns F34's facet meanings and FU10's canonical conversion;
this plan migrates their authoring/relational consumers.

The target has one grammar, one checked occurrence product, explicit immutable admission context
and real enforcement boundaries consuming shared predicates. Final immutable sources determine
admission, independent of edit history. An unused front door or global installer does not remain
because tests happen to exercise it.

## Decisions and interfaces

- A reusable document is keyed by bytes and all interpretation-bearing context actually consumed:
  document specification/identity, identity policy, package context and relevant parser/admission
  configuration. Unrelated manifest fields do not force unnecessary invalidation.
- Identity-policy and package-identity edits produce the same result as clean loading the final
  sources. Changed identity reparses/rebinds identity-bearing rows and attribution; binary
  wrappers receive the current document identity. No privileged incremental bypass remains.
- Authored/generated declaration text remains interchange. Checking creates admitted expression
  occurrences once, with role, declaration/document-relative location, bound/imported context
  and synthetic origin where applicable. Shared syntax never shares the wrong location.
- Extend the existing predicate grammar for logic operations and quoted paths. Specialization
  and compilation consume checked occurrences. Construct compiler-created expressions directly,
  not by formatting and reparsing names. AST handles remain local, not durable identifiers.
- Inject immutable validation context from the actual runtime/engine composition root.
  Generated/local construction receives an explicit local-validation context; runtime admission
  receives its actual planner/function/configuration owner. No first-wins ambient installation
  or test-only bind-defaults behavior supplies hidden meaning.
- Initialize memoized implementations per slot outside the global map lock. Keys include the
  context on which the implementation actually depends; reentrant construction cannot deadlock
  or reuse a foreign context's implementation. Different-slot nested construction is allowed;
  reentry into the same construction dependency chain yields a typed cycle refusal naming its
  implementation keys. Independent concurrent waiters share completion normally. Track the
  bounded wait dependencies between concurrently building slots in the context as well: before
  waiting, detect a cycle such as worker one building A→B while worker two builds B→A and refuse
  the implicated construction. Remove wait edges on completion/failure/cancellation.
- Keep shared Rust closure/reference/acyclicity predicates at actual physical admission and
  source publication. Delete their duplicate registry-invariant copies and the false claim of
  a global runtime SQL enforcement stage. Multiple legitimate boundaries may call one predicate.
- Delete inert requirement-planner/empty-policy wiring rather than activating it to justify
  retaining code. Delete test-only raw compiler Inputs/tracked front doors, unused whole-model
  structure preparation and unused structural projection. Preserve independently justified
  canon/codec mechanisms with current roles; a similar name is not evidence of legacy scope.

These choices avoid both a second grammar and a new universal admission framework. I consumes
the resulting context-complete products; it owns caching and source/mathematical identity splits.

## Packets

| Packet | Prerequisites | Responsibility | Status |
|---|---|---|---|
| <a id="h1"></a>H1 Context-complete document reuse | Current document owner | Make incremental and clean loading equivalent, including identity-bearing binary wrappers | done |
| <a id="h2"></a>H2 Checked occurrences and grammar | H1 | Parse once at check with complete attribution; migrate consumers and logic | done |
| <a id="h3"></a>H3 Explicit validation context | Current composition roots | Eliminate ambient installation and lock-held implementation construction | done |
| <a id="h4"></a>H4 Predicates and dead mechanisms | H2/H3; A1 | Share real boundary predicates/facet projections and remove unused front doors/enforcement claims | done |

H1 can correct reuse before the full grammar migration. H4 deletes mechanisms with no production
responsibility; it does not wait for I3's separate replacement of the actual production cache.
E1 remains the owner of consumed structural assessment.

### H1 — Reuse context

**Implementation vision.** A load receives final immutable package sources plus interpretation context. Its reusable product
contains rows, spans, current document association and recorded dependencies; the key covers source
content and the context actually used to create that product. An identity-policy change invalidates
identity-bearing interpretation even when source bytes match. An unrelated manifest description
need not. Binary buffers may remain shared while the current document wrapper changes, but old
identity-bearing rows cannot be kept by patching only an outer ID. Reuse returns a valid complete
product or reruns the affected stage, yielding the same admitted result/refusal as clean loading.

Replace the path/bytes/DocumentSpec-only predicate with the context needed for the operation.
A simple reparse/rebind when context changes is sufficient; only factor out context-free syntax
if it reduces actual duplicated work without weakening identity/attribution.

Focused controls load named-policy files without IDs, switch to explicit policy and compare with
a clean refusal; exercise the reverse change, package-identity changes and binary document IDs.
Retain unchanged-document owner reuse when its complete context is unchanged. Delete the old
incomplete predicate, not just an extra manifest special case.

### H2 — Expression occurrences

**Implementation vision.** An occurrence identifies document/declaration, expression role and structural position—for example
equation RHS, lower bound or third predicate operand. Span is revision attribution, not its sole
identity. The checked product contains AST, role, resolved declaration/import environment,
known type/physical obligations, dependencies and source/synthetic origin. Indexed/binding-dependent
obligations remain explicit for specialization instead of being falsely called fully instantiated.
Identical text may share syntax but not location or context. Specialization consumes the occurrence
with admitted bindings; compilation receives resolved nodes carrying that lineage. A malformed
repeated expression therefore reports each actual occurrence, not whichever copy was cached first.

Build the side product once at check, including equations, bounds, domain predicates, logic and
other expression roles. Inherited/overridden declarations must bind occurrences to their actual
context; identical text can share syntax storage while diagnostics name each occurrence.
Synthetic AST nodes retain an explicit originating operation/declaration.

Focused controls cover quoted logic paths, precedence, malformed expressions at multiple
locations, inherited context, generated expressions and repeated/indexed formals. Migrate every
consumed parse-at-use path, delete the independent logic grammar and format-then-parse builders.
Do not force a wire-format rewrite merely to share admitted ASTs.

### H3 — Composition roots and memoization

**Implementation vision.** The composition root creates immutable registry ownership, planner/function bindings,
configuration and predicate capabilities as an explicit validation context. Row/batch admission
receives it. Local construction uses a deliberately local context; a runtime obligation requiring
unavailable semantics refuses rather than falling back silently. Prepared implementations are
indexed by relevant context and contract. Map lookup allocates/finds a synchronized slot, releases
the map lock, and only then builds. Concurrent waiters can share completion, different-slot
construction can nest, and same-chain reentry reports the cycle keys. Construction order no
longer decides which registry semantics the process happened to install first.

Pass context explicitly through the owners that validate rows or construct derived implementations.
Pure local validation remains possible without a session/store. Runtime admission cannot silently
fall back to that weaker context. Remove public global installation and tests that depend on its
construction order.

Focused controls exercise two incompatible contexts, deterministic order independence and
different-slot construction, same-chain cycle refusal and independent concurrent waiters.
Include two concurrent constructors whose dependencies form A→B→A, so local recursion checks
alone cannot pass the test. Verify the global memo map lock is not held during build.
Remove replaced installers and their tests; keep behavioral admission controls against the new
explicit owner.

### H4 — Real authority and deletion

**Implementation vision.** A boundary supplies explicit declarations/rows to shared closure/reference/acyclicity predicates
and receives typed violations with affected identities. Physical loading and source publication
both call those operations; neither relies on the other having validated earlier. Field comparison
requests A1's declared purpose and preserves the separate directional annotation-restoration
contract. Numeric admission consumes A1's selected physical definitions/conversion. The admitted
result is then the sole production input to modeling and per-view structural assessment. There
is no second raw-input path, inactive requirement policy or imaginary global SQL validation stage
to suggest that a different authority already enforced these obligations.

Route field-meaning comparison and relational unit compatibility through A1's declared purposes.
Both inline and column paths consume A1's checked finite conversion; there is no H-owned competing
physical rule. G2 consumes the same predicate semantics for recorded witnesses.

Move shared closure/reference/acyclicity operations to their real Rust authority and delete the
registry duplicates. Retire the raw Inputs front door and its test-only tracked queries, unused
PreparedModeling.structure and structural Projection, and inert RequirementPlanner/provider-policy
wiring. Exercise real admitted package behavior when replacing tests that establish still-required
contracts; do not port tests whose only purpose was the deleted mechanism. Correct blueprint §4.6
and the obsolete structural/reuse descriptions with E/I.

Focused controls establish equivalent refusal at real enforcement boundaries, physical equality
under display-only changes, preservation of basis/reference distinctions and lawful defined units.
Use a scoped caller search to support deletion; caller counts alone do not establish semantic
replacement. The single consumed per-view structural owner remains.

## Authority and handoff

Update blueprint §4.6/§7.7/§22 and the relevant source-admission description through the design
route. A1 owns metadata-convention decisions. A new kernel checked-occurrence contract takes the
short ADR route; ordinary removal of unused code within current contracts needs no new ADR.
No source-proof manifest or new alignment tooling is introduced.

Hand I complete interpretation dependencies and occurrence attribution; hand A/C/D checked
expressions and context; hand G shared witness predicates. J's generated request shapes remain
separate from source-authoring and hydrated-document schemas.

## Consumed 25c prerequisite slice

**Implemented/Tested, 2026-10-01; scoped focused verification recorded in [25c Verification](25c-process-composition-and-conservation.md#verification):** Context-sensitive document/parser reuse and identity-bearing binary wrappers are migrated in their runtime/compiler consumers. Checked declaration/field occurrences retain ASTs, physical admissions, dependency and index context. C1/C5 consume precise retained process fields. General prior specialization/body consumers and the remaining grammar/validation migration are still H2/H3/H4 work. The maintainer authorized only this required slice and its complete affected consumer migration. This packet remains partial; [25c](25c-process-composition-and-conservation.md) owns the slice evidence.

## Consumed 25d prerequisite slice

**Implemented/Tested, 2026-10-01:** H2's required mathematical-realization/response
slice is integrated. Implicit relation/branch/operational selector syntax, rendered documents and checked occurrence consumers carry semantic anchors, admitted settings and neighborhood predicates. The seed explicitly declares its operational density selection. Wider document reuse, expression/admission migration and H3/H4 ownership remain open.
[25d Verification](25d-mathematical-realization-and-response.md#verification) owns commands,
conditions, composite results and limits; this does not close the enclosing packets.

## Consumed 25f prerequisite slice

**Implemented/Tested, 2026-10-01:** Study requests now pass through source-owned typed
admission. Compiler-owned demand resolves submitted case overlay paths once; admitted bindings
retain canonical members, full quantity meaning, source revision and physical context. Horizon
input/prior/trajectory values are admitted once against their owning physical targets; replay
validates canonical entries and exact operation destinations against pinned source. Neither
executor reconstructs a scalar study overlay or converts units again. Focused source/physical
admission and replay controls are owned by
[25f Verification](25f-studies-diagnostics-and-continuation.md#verification). Wider H1/H2
document/grammar migration and H3/H4 validation-context/predicate consolidation remain open;
cross-owner journeys remain 25k.

## Consumed 25g prerequisite slice

**Implemented/Tested, 2026-10-01:** Current declarations and verified recorded witnesses now
feed one predicate compiler through an explicit domain environment. Recorded providers retain
actual field provenance; consumer admission is directional and cannot grant exact writer
admission. Obsolete protobuf predicate serialization/decoding is removed. The focused controls
and limits are owned by [25g Verification](25g-durable-contract-evolution.md#verification).
This completes only G's required H4 predicate/admission slice; wider authoring, validation-context
and inactive-authority consolidation remains open.

## Execution and evidence

**Implemented/Tested, 2026-10-02:** H1–H4 are complete within this packet's functional scope.
The dated prerequisite records above retain their original scope and evidence. ADR-0150 owns
the checked-occurrence, explicit-context and owned-reuse contract; the
[series coordinator](25-design-remediation.md) owns finding dispositions and decision dependencies.
The focused controls below establish authoring/admission behavior and retained production consumers.
Whole-product integration, formatting, lint and performance qualification remain in
[25k](25k-integrated-qualification-and-closure.md). No broad qualification is claimed here.

## Verification

**Tested, 2026-10-02:** Local Linux checks used the pinned nightly, locked dependencies and explicit
`pse-relations/force-validate` through the recipes. Failure and warning targets remain zero.
Unit selections used Nextest's default profile and concurrency unless stated otherwise; skipped
cases were not exercised. Commands ran inside `direnv exec .`. The final compiler seed/graph
selection used the licensed environment, memory-cap wrapper and `--test-threads 1`.

| Command | Result against zero failures | Established scope |
|---|---|---|
| `just unit-package pse-authoring 'test(logic_uses_shared_quoted_paths_counts_and_precedence) or test(typed_callees_preserve_quoted_indexed_receivers_and_partial_paths) or test(exact_payload_field_spans_keep_repeated_text_distinct)'` | 3/3 passed; 76 skipped | Logic uses the shared predicate machinery; typed quoted/indexed callees and partial selectors; parser-owned exact repeated-field spans |
| `just unit-package pse-modeling 'test(occurrences::tests) or test(kernel_types::indexed_function_types) or test(kernel_types::indirect_function_calls) or test(kernel_specialization::each_instance_realizes) or test(kernel_specialization::presets_bind) or test(kernel_specialization::annotations_resolve) or test(kernel_specialization::fixture_diagnostics_resolve) or test(physical_reconstruction_specializes_exact_maps_and_selected_response_witness) or test(process_contract_unannotated_children_retain_nominal_connection_contracts) or test(process_contract_inventory_event_transfer_requires_a_local_numeric_guard_member) or test(process_contract_ftpz_and_fphz_keep_explicit_reference_species_and_derived_observations)'` | 19/19 passed; 208 skipped | Exact malformed/repeated/inherited attribution, actual production field inventories, quoted space/literal-dot specialization and recursion refusal, lexical shadowing, indexed obligations, retained annotation/fixture/form consumers, reconstruction/response callees and nominal child connection contracts |
| `just unit-package pse-schema 'test(implementation_cache::tests)'` | 4/4 passed; 103 skipped | Construction outside the map lock, distinct-slot nesting, same-chain and cross-worker cycle refusal, concurrent waiter sharing and interruption wakeup |
| `just unit-package pse-relations 'test(schema_preparation_) or test(actual_session_function_bindings_are_prepared_in_each_owner) or test(sql_checks_require_an_explicit_native_binding)'` | 4/4 passed; 35 skipped | Actual incompatible native function owners, local-context refusal for SQL, nested schema preparation and same-chain/cross-worker cycles |
| `just unit-package pse-rules 'test(explicit_registration_and_parent_closure_refuse_affected_identities)'` | 1/1 passed; 42 skipped | Shared typed registration, field agreement, missing-parent and parent-cycle violations retain affected identities |
| `just unit-package pse-runtime 'test(document_reuse_) or test(source_and_physical_boundaries_refuse_the_same_missing_registration) or test(source_and_physical_boundaries_refuse_the_same_package_closure_and_cycle) or test(complete_source_package_context_retains_admitted_external_headers) or test(pse_source_uses_generated_rows_and_original_ranges) or test(job_request_identity_independent_of_key_order) or test(modeling::applicability_tests::) or test(report_transfer_context_retains_actual_owner_order_and_direction) or test(pushdown_columns_are_the_registry_columns)'` | 14/14 passed; 217 skipped | Clean/incremental identity-policy equivalence, parser-owner reuse, changed package/binary identity and tighter parse-budget refusal; actual source/physical closure/reference/cycle equivalence; admitted external physical-package header context and refusal of a mismatched submitted header; explicit runtime field codecs and owner/direction preservation |
| `just unit-package pse-compiler 'test(conditional_unit_recycle_reference_specializes_actual_unit_boundaries) or test(kernel_flow_projection_preserves_ports_isolates_and_explicit_tear_policies)' --test-threads 1` | 2/2 passed; 229 skipped, on the final frozen source | Retained authored seed specialization and actual unit-boundary/graph contracts after typed callee and contextual child-reference migration; no numerical solver qualification |
| `just unit-package pse-structural 'test(complete_inventory_isolates_coupling_partial_and_provenance) \| test(empty_rectangular_and_structural_not_numerical_rank)'` | 2/2 passed; 45 skipped, coordinator-run | Consumed per-view inventory, isolates, coupling, provenance and rectangular structural rank after deleting unused Projection; numerical rank remains distinct |
| `just check-package pse-engine` and `just check-package pse-catalog` | Passed, all targets, zero source errors/warnings | Owned engine/catalog production, fixture and generated-context consumer joins; the coordinator owns the final cross-workspace compile after generation |

These are composite receipts, not initially clean runs. Compile retries repaired migrated context,
owned-batch and fixture signatures. Behavioral failures exposed quoted primary-call parsing,
missing word-token field ranges, an obsolete function-kind purity whitelist, synthetic coordinate
slot ownership and unannotated-child nominal lookup. Their replacements passed the final controls.
The new child fixture also needed its explicit connectivity policy; the operational-table fixture
needed its actual runtime context instead of a local context. No acceptance condition or failure
baseline was relaxed. Final Rust source compilation emitted no warnings. Runtime/catalog commands
still emitted Cargo's upstream future-incompatibility advisory for `proc-macro-error2 v2.0.1`;
that advisory is recorded, not waived as a quality baseline, and remains with comprehensive 25k
qualification.

**Implemented, scoped caller inspection:** Authored/modeling/schema/relations/engine/columnar/rules/
catalog/runtime source and owned fixtures no longer contain `RequirementPlanner`, ambient
`for_registry`/`bind_defaults` installers or `DefaultValidationContext`. Production modeling parses
DSL text at checked occurrence collection, newly submitted demand-name admission and the separate
unit-spelling boundary; consumed authored-expression paths use retained ASTs. The independent logic
tokenizer and formatted synthetic-expression parsing are deleted. The coordinator/compiler owners
removed raw Inputs/tracked test front doors, unused whole-model structure preparation and unused
structural Projection. `PreparedCase.structure` and consumed specialization structure remain.
This inspection supports the named deletions; it is not a full static or product qualification.

**Interface-checked, 2026-10-02:** Final cross-plan regeneration and workspace/all-target
compilation passed after the fixture and benchmark consumer cutover. The composite integration
receipt and linked Python boundary checks are owned by
[25j Verification](25j-generated-boundaries-and-library-consolidation.md#verification).
This establishes source integration, not the unexecuted 25k qualification campaign.

## Outcome (recorded after implementation)

### What was built

**Implemented:** H1–H4 now use one checked source product and explicit admission ownership.
Document reuse carries complete interpretation dependencies, parser field ranges and current
identity-bearing wrappers, and re-admits projections under the supplied runtime context.
Checking retains role/position, syntax, exact authored attribution, lexical/import context,
physical admissions, dependencies and pending indexed/binding obligations. Consumers use precise
occurrences; logic extends the existing DSL machinery and synthetic operations construct ASTs.
Named calls and partials carry typed paths, and decoded segment lookup preserves quoted dots,
spaces and lexical shadowing through function dependency, recursion and specialization consumers.

Validation context comes from the actual factory/session/runtime owner and is threaded through
row, view, batch, collection, loader and catalog boundaries. Deliberate local construction refuses
native SQL obligations. Implementation and schema-preparation slots share the construction owner,
build outside map locks, share normal completion and refuse same-chain/cross-worker cycles with
typed causes. Global installers and the inert requirement-planner/provider-policy bridge are
removed; effects, settings, byte limits and native execution-contract obligations remain.

Source publication and physical admission independently consume shared package and registration/
parent-closure predicates over actual facts. Complete package-header context may include an
already admitted physical package omitted from the submitted model documents; submitted headers
must agree with that context. Partial loading remains possible before the complete boundary.
Duplicate registry-invariant copies and imaginary global SQL enforcement are removed. Compiler
and structural deletions preserve the consumed per-case/per-view witnesses.

**Tested:** The Verification controls establish these mechanisms and the retained production
seed/graph consumers under the stated local conditions. Integrated scientific/storage/Python
journeys and performance are outside this packet's evidence.

### A mistake made and corrected

Moving callees from strings to typed paths initially left old string-rendered resolution and a
function-kind whitelist in the closure checker. That lost quoted names in recursion/dependency
lookup and misclassified response and generated map-slot calls as captured runtime state. The
retained seed exposed those false refusals. Resolution now traverses decoded atomic segments,
respects lexical formals and uses checked callable/operation ownership, including slots of an
immutable package-owned coordinate map. Quoted-dot nonalias, recursion, actual specialization and
the retained seed/graph controls pass; no general unknown-reference exemption was introduced.

Strict source-range checking also exposed a missing word-token producer for operational anchor
fields. Recording that range in the actual parser fixed the attribution product instead of
falling back to whole declarations for authored fields.

### Deviations from the plan, deliberate

**Implemented:** Internal `NamedCall.name` and `Partial.function` now hold typed paths directly,
rather than retaining strings plus another callee-parsing product. Their AST interchange is not a
durable format; authored grammar and the separate source/hydrated-document schemas remain distinct.
The explicit complete package-header context preserves the existing admitted-physical-package
join while enforcing closure at both real boundaries. Both choices follow ADR-0150 and the
coordinator's reviewed implementation direction.
