---
title: "25h: Authoring and admission ownership"
status: in-progress
date: 2026-09-30
adrs: []
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
| <a id="h1"></a>H1 Context-complete document reuse | Current document owner | Make incremental and clean loading equivalent, including identity-bearing binary wrappers | partial: 25c prerequisite slice |
| <a id="h2"></a>H2 Checked occurrences and grammar | H1 | Parse once at check with complete attribution; migrate consumers and logic | partial: 25c prerequisite slice |
| <a id="h3"></a>H3 Explicit validation context | Current composition roots | Eliminate ambient installation and lock-held implementation construction | planned |
| <a id="h4"></a>H4 Predicates and dead mechanisms | H2/H3; A1 | Share real boundary predicates/facet projections and remove unused front doors/enforcement claims | planned |

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

## Execution and evidence

The consumed 25c prerequisite slice above is **Implemented**; its focused evidence is owned
by 25c. The remaining packet scope and expected benefits are **Proposed**. No full-packet
completion or new broad product qualification is claimed. The [series coordinator](25-design-remediation.md)
owns finding dispositions and decision dependencies. Packets compile affected owners, run focused
behavioral checks with explicit force-validation, regenerate changed declarations, and immediately
delete replaced code, callers, obsolete tests and fixtures. No shims or parallel production paths remain.
Full integration, formatting, lint and performance qualification run once in
[25k](25k-integrated-qualification-and-closure.md), after the series' functional scope is complete.

Use current recipe-owned checks such as `just check-package <pkg>` and
`just unit-package <pkg> <filter>`; select isolated tests rather than broad suites hidden under
a unit label. The acceptance scenarios above define what those tests must establish, not claims
that tests with particular names already exist. Cross-owner scientific/storage journeys are authored
with the functional work and executed in 25k. Record state, decisions and next steps during work;
record actual commands, conditions and failures against zero in the final qualification evidence.

## Outcome (recorded after implementation)

### What was built

Full-plan closure remains outstanding. The implemented 25c prerequisite slice and its
remaining boundaries are recorded above; 25c owns its focused execution evidence.

### A mistake made and corrected

Record an actual implementation correction, not a hypothetical planning example.

### Deviations from the plan, deliberate

None recorded. A changed architectural decision follows its owning ADR/design route.
