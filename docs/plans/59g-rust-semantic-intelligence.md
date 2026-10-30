---
title: "Plan59g: Rust semantic intelligence"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, environment, diagnostics]
review_sources: [docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md]
---

# Rust semantic intelligence

## Purpose and assessed foundation

**Proposed:** give an agent a direct route from a source question to relevant semantic
evidence, then to a selected expansion or edit proposal. This companion owns analysis
sessions and semantic producers for the [code-intelligence review](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md):
O01–O04, O07–O09, O13, the analyzer portion of O10 and the semantic inputs to O16.
[D01](59d-knowledge-and-workflow-verbs.md#d01) owns assembled investigation packets;
[B02](59b-runs-and-continuity.md#b02) owns original compiler diagnostic extraction.
The coordinator owns source opportunity dispositions and combined completion.

The source baseline is `a569b52e4`, 2026-10-29. The review's preceding end-to-end library
research and focused code assessment are retained at their original strength. Production
execution remains stopped; none of this document's packages is implemented by authoring it.
Core 3.4, efficiency heuristics 1.0 and process-simulator 1.5 apply through the binding.
This is developer tooling, not a new scientific or numerical interpretation owner.

Operator-confirmed RC01–RC05 govern planning. The selected target includes reusable owned
analysis sessions, useful preview-only semantic editing, and conditional application after
G08 establishes a proportionate concurrency/publication route. Existing MCP/editor/Codex
configuration exclusions remain. There is no requirement to install a bridge or share an
editor's stdio connection.

Current `pse-host` owns preparation, process placement, authenticated control and settlement;
the correctness adapter owns Cargo selection and force-validation. The planned A03/A04/A05
contracts supply output, resource-specific storage and protected prepared generations.
C07 supplies shared selection. These are reusable foundations, but their planned readiness
is not implementation evidence. In particular, B03 must deliver actual service-entry ownership
before a cross-invocation analyzer session can rely on it.

## Target, choices and shared boundaries

### Questions and presentations

Use `pse code` as the semantic command family within A02's single tree. G01 fixes the supported
subcommands against the pinned backend. The intended operations are `explain`, `symbols`,
`refs`, `calls`, `implementations`, `macro`, `tests`, `candidates` and `edit`, with explicit
position/range or qualified-symbol subjects. Session `prepare`, `status` and `stop` belong
to this same discoverable family; help/status do not prepare it. D03 `outline`/`show` remain
server-free syntactic reads, not aliases for semantic resolution.

An `explain` response combines available declaration/signature/type presentation, definition,
trait origin and a short source excerpt. A range adds only selected annotations: intermediate
types, captures, adjustments, argument names or relevant exits. Additional references, calls,
tests, docs and edits are next actions, not automatic work in every request. A combined response
aims for approximately 800–1,200 tokens using A03 detail budgeting; this is not a new tokenizer,
byte truncation contract or permission to omit required status/context.

The A03 envelope retains provider, selected package/crate-instance/target/features, synchronized
analysis generation, source locations, availability and semantic scope. It distinguishes no
match, incomplete model, preparation required, unsupported capability, canceled/stale analysis
and projected detail. Lists report known shown/omitted counts or an unknown total. Search scope,
completed examination and displayed subset remain separate. A bounded response can require
substantial reference search; presentation limits do not establish cheap execution.

CLI file positions use documented one-based line/column input. G03 establishes Unicode column
semantics and performs explicit conversion to the negotiated LSP encoding and source offsets;
it must not assume UTF-16 equals byte offsets. Result handles and lazy resolutions are bound
to their analysis/source generation and reacquired after invalidation. They are not permanent
symbol identities.

### Backend and physical realization

Prefer an owned rust-analyzer process over stdio LSP for navigation, references, hints,
completion and code actions. It supplies workspace management and native incremental analysis
without embedding compiler frontend internals in ordinary host handlers. G01 checks actual
capabilities of the selected pinned binary, including Rust-specific extensions. Native
`ra_ap_ide`/HIR integration remains eligible where a material selected field or operation is
unavailable over that route. It is a deliberate scoped choice with its own VFS/configuration/
macro/invalidation responsibilities, not an automatic fallback or host-ceiling expansion.

Expose typed semantic fields only where the backend supplies them. Hover or HIR/MIR debug text
may be rendered as an explanation/artifact; parsing it into an authoritative typed model is
not the design. Unknown inference or ambiguous resolution remains explicit. Rust-analyzer
does not establish compiler borrow correctness or a clean Clippy result.

Use analyzer-native Salsa queries, snapshots and invalidation. One compatible session per
checkout/configuration reuses workspace preparation across ordinary queries. A session owns
its analyzer/proc-macro children through the existing host lifecycle; it does not create a
database, durable queue, distributed lease protocol or resumable query system. Explicit stop
ends the session after actual child settlement. G02 uses an explicit-session lifetime until
stop or authenticated checkout teardown; an idle eviction policy is optional later tuning,
not a required timer added to the first implementation.

Canceling a client request must not destroy the session. Source/configuration changes may
cancel all outstanding old snapshots; return canceled/stale and permit reacquisition rather
than joining results from different generations. Analysis-generation coherence does not
claim an atomic cross-file filesystem capture. Pending reload/build/proc-macro state and
observed source mismatch remain visible. Do not hash the whole repository on every query.

Cold preparation is explicit. Build scripts/proc macros can require Cargo work even with
check-on-save disabled. Their supported preparation command must enter the existing environment,
selection and placement route and retain Cargo JSON behavior. Do not add another automatic
check loop. Restricted analysis can be useful with declared missing preparation. Syntax reads,
original-run diagnosis, help and stop remain usable when semantic preparation is unavailable.
Use ordinary maintainer-selected workstation capacity; no smaller CPU partitions, arbitrary
serial lanes or new RAM caps are implied.

Concrete source leads: [analyzer overview §§3–4, 8–10](../libraries/rust_analyzers/rust-analyzer.md),
[IDE overview](../libraries/rust_analyzers/ra_ap_ide.md),
[HIR overview](../libraries/rust_analyzers/ra_ap_hir.md),
[upstream API boundaries](https://rust-analyzer.github.io/book/contributing/architecture.html),
[LSP extensions](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/lsp-extensions.md).
The review checked development interfaces; exact pinned extension/native signatures remain G01 work.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [G01](#g01) | planned | 59g | Pinned protocol coverage, concrete query/result design and native-gap decisions |
| [G02](#g02) | planned | 59g | Explicit preparation and reusable authenticated analysis session ownership |
| [G03](#g03) | planned | 59g | Selected context, source synchronization, positions and stale-result handling |
| [G04](#g04) | planned | 59g | Resolution, discovery and selected implicit-semantic explanations |
| [G05](#g05) | planned | 59g | References, bounded calls/traits and macro/source provenance |
| [G06](#g06) | planned | 59g | Candidate tests and contextual code-construction discovery |
| [G07](#g07) | planned | 59g | Source-bound rename, assist and SSR previews |
| [G08](#g08) | planned | 59g | Apply investigation concluded; implementation only if its publication criteria hold |
| [G09](#g09) | planned | 59g | Integrated semantic consumer, packet producers and lifecycle acceptance |

<a id="g01"></a>
## G01 — Pinned capability and integration design

**Consumes:** the accepted review target, A03 output meaning, A05 preparation design and the
current pinned analyzer/toolchain. This design inquiry does not wait for all of A–F to execute.

Map each intended query and decision-relevant result field to standard LSP or negotiated
rust-analyzer extension requests. Verify selected binary behavior where source inspection
leaves consequential uncertainty. Preserve recognizable native leads: `goto_definition`,
`goto_type_definition`, `hover`, `signature_help`, `find_all_refs`, call hierarchy,
`inlay_hints`/lazy resolution, completion, rename/code actions and macro/test extensions.
Choose exact command spellings, supported scopes and result schemas with A02/A03/A06.
Do not invent a server-side filter merely because a CLI option would be convenient.

For required structured capture/adjustment/trait-origin data unavailable through LSP, compare
selected native IDE/HIR integration with a faithfully rendered or explicitly unsupported view.
Use native integration only for an identified gap whose utility justifies full ownership cost;
route any crate/contract changes before dependent implementation. Missing coverage does not
silently erase the selected opportunity or cause raw-string semantic parsing.

**Acceptance:** a bounded design identifies actual pinned mechanisms, selected response shapes,
unsupported/conditional fields and the affected consumer route. Probes, if needed, distinguish
an inspected signature from a working invocation. The default LSP route and any native branch
have explicit decision outcomes; no performance or whole-workspace completeness claim follows.

<a id="g02"></a>
## G02 — Preparation and reusable sessions

**Consumes:** G01, working A03/A04/A05 preparation/generation-use slices and B03's authenticated
service-entry/settlement slice. Generic session ownership does not require full B02 test
extraction, B04 bundles or product F readers.

Explicit preparation selects compatible analyzer/toolchain/configuration and required build/
macro inputs. Start or connect to one authenticated checkout/configuration session. Retain its
prepared generations while actually used, keep status/readiness available separately from
query success, and stop using original ownership records even if current host sources break.
Multiple CLI clients use the session's adapter; they do not independently write to one stdio
stream or assume editor attachment. Bounded batches share the same engine without becoming a
second persistent analysis backend.

Configure automatic checking off. Qualify `cargo.buildScripts.overrideCommand` and proc-macro
preparation against the repository's Cargo JSON/environment requirements. Restrict or declare
missing macros when preparation is unavailable rather than launching unadmitted work.
Independent teardown attempts retain their failures and prove original child drain before
releasing pins. E01 removal sees sessions as live users.

**Acceptance:** two separate client invocations reuse one compatible session; a differing
configuration does not silently reuse it. Canceling one request leaves another query/session
usable. Explicit stop settles analyzer and macro children, including a delayed child, and a
failed settlement remains incomplete. Missing tools, pending preparation and broken current
sources leave status/stop/source-only reads useful. No new scientific authority is acquired.

<a id="g03"></a>
## G03 — Context and source synchronization

**Consumes:** G02 and C07's package/target/features context slice. Source semantics need that
working slice, not full fixture preparation, actual test binaries or a feature-matrix campaign.

Maintain the analyzer's file/configuration inputs for edits, additions, deletion, generated
outputs, manifest changes and source-root changes. Select the crate instance when one file is
compiled in multiple contexts. Report analyzed cfg/test/dependency context and differences from
actual compiler-unit selection. Use native reload/invalidation and coherent result generations;
do not construct another fine-grained semantic cache or infer revision from Git HEAD alone.

Implement position conversion against the negotiated encoding with source-bound locations.
Bind lazy hints, completion edits, continuations and edit proposals to their input revision.
Report pending reload/macro state or observed races. A changed revision invalidates dependent
handles; native old-snapshot cancellation is legitimate and is distinct from session failure.

**Acceptance:** non-ASCII positions select and edit the intended token; add/delete/body/manifest
and feature changes update or invalidate results. Two crate instances cannot be conflated.
An old hint/edit handle refuses or reacquires current context; no mixed-generation response is
called coherent. Watches are not advertised as an atomic filesystem snapshot.

<a id="g04"></a>
## G04 — Explain selected code

**Consumes:** G03 and A03/A06 presentation/discovery slices; D03 supplies optional source-only
expansion but is not a reason to start semantics for syntactic reading.

Deliver O01/O02 symbol discovery and call/range explanation. Compose definition, available
signature/type presentation and origin, then selected inlay/related-code annotations. Useful
categories include intermediate chain types, closure returns/captures, binding modes,
argument names, generic arguments and implicit adjustments. Resolve hint details lazily.
Native leads include HIR `type_of_expr`, `expr_adjustments`, `Closure::captured_items`,
`capture_types`, `fn_trait`, future output and iterator item, only where G01 selected support.
An analyzer capture annotation is not a borrow-check result or measured allocation.

**Acceptance:** same-spelled methods resolve to distinct declarations; ambiguity and unknown
inference survive. An iterator/async chain yields selected useful annotations and one lazy
expansion without dumping the file. The response preserves mandatory context under a small
detail budget; source-only reading still works for an incomplete file.

<a id="g05"></a>
## G05 — Explore references, calls, traits and macros

**Consumes:** G03. D06 supplies generated-output owner enrichment; the basic semantic query
does not wait for every generated map consumer to migrate.

Deliver O03/O04/O13 as explicit expansions. Group semantic references by file/containing item
with optional test/import/generated categories, retaining pattern/type/constructor uses.
Call views default to one hop and mark cycles/repeated nodes; never enumerate all paths.
Keep relationship kinds and source scope. Trait declarations/supertraits, related impls,
selected invocation resolution and a predicate question are separate views. Native
`Impl::all_for_type` excludes blanket impls and is shallow; it cannot establish universal
applicability. Indirect dispatch/callback/FFI edges remain unknown where not supported.

Macro expansion returns selected text plus available origin and invocation fallback. HIR
descent/original-range mechanisms can enrich provenance where selected. Missing preparation
is not empty expansion. Registry-generated files and virtual macro expansion have different
owners; neither permits writing generated output. D01 may combine these views with package
reach, but semantic absence never narrows authoritative conservative test selection.

**Acceptance:** unrelated same-name symbols are excluded from resolved references; scoped
absence is not reported as no external users. A recursive call neighborhood terminates with
cycles explicit. A blanket implementation is not falsely ruled out. One generated impl leads
to the editable invocation/generator or explicitly reports unavailable precise mapping.

<a id="g06"></a>
## G06 — Candidate tests and code construction

**Consumes:** G03. Actual runnable handoff consumes C05/C07's working discovery/selection;
candidate navigation and completion can ship before full C04 fixture support.

Deliver O07/O08: `related_tests`/runnable discovery as source candidates and context-sensitive
completion for receiver methods, imports, constructors and expected call arguments. Return
candidate location/signature, required imports and insertion preview; resolve selected
completion edits/docs only on demand. Candidate ranking is heuristic. O07's tests are neither
measured coverage nor complete nextest enumeration; send a selected candidate to the original
test owner rather than directly executing analyzer-produced argv.

Support useful completion without interactive editor snippets. If a chosen insertion has
snippet placeholders, expose them as unresolved editing intent rather than silently inventing
implementation. Term synthesis remains the conditional inquiry below.

**Acceptance:** navigate from production code to a relevant test and select it through C's
actual route. Missing candidates do not suppress affected selection. At an incomplete call,
discover a trait method and required import; its chosen insertion remains source-bound and
compilation through the ordinary owner is distinct from candidate generation.

<a id="g07"></a>
## G07 — Semantic edit previews

**Consumes:** G03 and the selected G01 rename/code-action/SSR capabilities. D06 protects
generated-owner paths in presented next actions; G07 itself changes no source.

Deliver O09 action discovery followed by selected multi-file diff generation. Use native
`prepare_rename`, `rename`, `will_rename_file`, `assists_with_fixes`, lazy
`AssistResolveStrategy`, `SourceChange` and semantic SSR where the pinned surface supports
them. Record original source identity, every affected file/filesystem operation, unsupported
macro/configuration regions and required snippet handling. Enumerating actions should not
eagerly calculate every edit. A pattern rule's resolution context matters; do not substitute
regex or syntax-only replacement while calling it semantic SSR.

Preview rename, import/match/trait-member assists and repeated resolved-call replacement.
Preserve captured arguments and report changed effect/evaluation-order obligations for review.
Returned edits are proposals, not an equivalence proof, applied transaction or automatic fix.

**Acceptance:** rename changes the intended entity without unrelated names; a scoped SSR
rule excludes a similarly spelled different function. A chosen assist shows all files and
unresolved placeholders. Source change invalidates the proposal. Preview performs no edits,
generation, compiler check or commit.

<a id="g08"></a>
## G08 — Conditional application decision

**Consumes:** G07 and D06's protected/generated ownership contract. This decision is distinct
from D09's Git-index transaction: a private Git index does not make source writes atomic.

Investigate an explicit apply action for selected proposals. Compare existing patch/file
publication mechanisms and isolation/cooperative coordination against actual concurrent editors.
Test the chosen candidate's stale-source refusal, file moves/creation, race between preflight
and write, mid-application failure and preservation of another agent's changes. Preflight
alone is insufficient and rollback must never overwrite subsequent unrelated content.

Adopt only if a proportionate mechanism supports a clearly declared concurrency boundary,
pre-write stale detection, affected-file ownership validation and truthful partial-failure
reporting/recovery. Do not promise crash-atomic multi-file filesystem updates without a real
mechanism. The command must distinguish no changes, complete application and identified partial
application; it must never automatically retry a possibly applied edit or silently run checks.
Compilation/targeted tests are separate explicit next actions under the existing policy.

If these criteria hold, implement and verify the selected apply route alongside its direct
consumers. Otherwise conclude with preview-only delivery, the concrete rejected alternatives
and a trigger: an actual repeated edit workflow for which a newly available publication or
isolation mechanism resolves the named gap. No elaborate general editing transaction system
is required to complete a justified negative inquiry outcome.

**Acceptance:** a written decision records mechanism/evidence and affected targets. If adopted,
the race, protected path and mid-application cases exercise actual writes and preserve exact
partial outcomes. If not adopted, no apply command is advertised as available; G07 remains
usable and the coordinator retains the conditional disposition rather than claiming apply.

<a id="g09"></a>
## G09 — Consumer composition and qualification

**Consumes:** selected working G04/G05/G06/G07 producers, D01 packet composition, B02 original
diagnostic source-generation references and A06 discovery. G08's conditional apply outcome
does not block initial semantic reading. Full H/I capabilities are not prerequisites.

Supply D01 with source-linked selected views for O16; keep packet selection/presentation in
that owner. Join an original compiler diagnostic to its matching source generation or expose
the mismatch. IDE-native diagnostic/failed-obligation views retain provider and unknown versus
contradiction; `full_diagnostics` is not actual Cargo/Clippy execution. F may offer a source
follow-up from runtime evidence but a static caller is not a propagated execution edge.

Exercise a real agent journey: explain an ambiguous invocation, inspect one reference/caller,
read a candidate test, discover an unfamiliar method/import and preview a semantic rename.
Show useful next expansions within the shared budget. Change source/configuration and repeat
without silently reading stale results; cancel one request, then stop and settle the session.
E03 communicates working verbs and D07 derives reference content as each producer ships.

**Acceptance:** the actual CLI journey demonstrates selected meanings, source mismatch and
missing preparation behavior, alongside independent expected resolution/reference controls.
Integrated lifecycle checks observe real children, not merely a mocked stop reply. No solver
or scientific correctness claim follows. Record exact scope and zero-baseline results at the
package/series qualification owner; no speed benefit is claimed without measurement.

## Conditional capabilities and deliberate limits

| Capability | Disposition, owner and observable trigger |
| --- | --- |
| Rich native HIR/IDE fields or embedding | G01 investigates when a selected O02/O04/O13 result cannot be faithfully exposed through pinned LSP. Adopt only the scoped gap with full workspace/input ownership and required decision route. |
| Tolerant source parsing | D03 remains syntax-only. Reconsider `ra_ap_syntax`/Rowan with G01/D03 when a real incomplete-file read cannot provide useful structure; no semantic session prerequisite. |
| Term search | G06 deferred experiment when an actual expected-type construction question is not served by methods/imports. Use native fuel, report exhaustion/non-exhaustiveness; no synthesis service prerequisite. |
| StaticIndex, SCIP/LSIF | G01 deferred alternative for an actual immutable-revision browsing/export consumer. Require snapshot/configuration identity and regeneration semantics; no second live graph by default. |
| Full semantic token/highlight export | Not selected for text-agent output. G04 may add category-filtered source annotations; full token export waits for an actual renderer needing it. |
| Tiny editor actions | No standalone CLI family. A useful action can enter G07 through the selected native assist; enter/join-lines/brace actions do not create initial obligations. |
| Analyzer status, RA_LOG/RA_PROFILE, analysis-stats | G02 readiness/status is selected; detailed native tool diagnostics are explicit investigations when the analyzer itself fails or is slow. No routine full-workspace profiling campaign or new monitoring stack. |
| Analyzer MIR/layout/interpretation | Compiler/representation questions route to 59i. Analyzer artifacts remain labeled as analyzer-derived; experimental interpretation is not trusted general execution. |

## Verification and handoff

**Proposed verification:** packages compile touched implementation closures and run targeted
behavioral tests, with force-validation on actual Arrow-consuming correctness work. G01's
contract checks do not count as G02/G09 integrated delivery. Scope-end static/manual checks
and relevant integration journeys run once with the coordinator's qualification, not for every
package or document checkpoint. Test ambiguity, Unicode, unavailable macros, revision races and
native settlement where those semantics are at risk; schema snapshots alone are insufficient.

**Current state:** all G packages are planned. No analyzer/service started, dependency installed,
product code changed or functional/performance test run by this authoring change. The next
design work is G01; working session delivery additionally waits for the named A/B slices.
The coordinator remains the only source-opportunity disposition owner and records completion
only with the selected consumer and required evidence.
