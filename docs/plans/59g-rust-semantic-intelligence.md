---
title: "Plan59g: Rust semantic intelligence"
status: complete
date: 2026-10-29
doc_role: plan
doc_retention: while-dependent
doc_retention_reason: "Retained complete series at maintainer request; Plan58, Plan60, proposed ADRs and qualification references consume its evidence."
doc_retirement_trigger: owner-release
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, design, libraries, environment, diagnostics]
review_sources: [docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md, docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md, docs/functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md]
---

# Rust semantic intelligence

**Complete:** selected delivery is closed; the [final Plan59 conclusion](59-agent-cli.md#current-state-and-next-step)
records the worktree-removal deferral and qualification limits. Earlier targets and dated
checkpoints below retain their original context; package tables and Outcomes record delivery.

## Current extension — 2026-10-31

**Implemented; targeted Tested:** G10–G12 extend the completed G01–G09 baseline with
name-first discovery and the selected immutable native IDE/HIR snapshot, carrying the
[search review's O03/O07/O10](../functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md).
The maintainer accepted `search`/`inspect`/`propose` hard cutover and compact shared
provenance/streaming on 2026-10-31, preserving identity, outcome, coverage and every useful
replaced read. Those decisions supersede earlier public-family and repeated-envelope target
wording below; completed rows and their evidence retain their original scope. K owns the shared
query/result contracts, A10 their governing publication and A11 any selected provider decision.
G11 selected the native branch and A11 published it before implementation. The selected
runtime and component evidence below is distinct from the coordinator's assembled
[P02 qualification](59-agent-cli.md#p02).

G11's SSR non-adoption and revisit conditions remain explicit; no SSR-search provider is
fabricated from rewrite output. The selected G12 branch and G10's existing-LSP route retain
separate prepared contexts. Neither gates ordinary lexical/syntax discovery.

## Purpose and assessed foundation

**Proposed:** give an agent a direct route from a source question to relevant semantic
evidence, then to a selected expansion or edit proposal. This companion owns analysis
sessions and semantic producers for the [code-intelligence review](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md):
O01–O04, O07–O09, O13, the analyzer portion of O10 and the semantic inputs to O16.
Focused producer views expose their own bounded answers and typed next actions;
[B02](59b-runs-and-continuity.md#b02) owns original compiler diagnostic extraction.
The coordinator owns source opportunity dispositions and combined completion.

The review source baseline is `a569b52e4`, 2026-10-29. Implementation starts from
`49aaeb3f8` and the shared original-native observation/peer-custody helpers. The review's
preceding library research remains evidence at its original strength; working implementation
and its focused acceptance are recorded below, independently from series integration.
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

Use `pse code` as a discoverable family with explicit provider authority; [59j](59j-source-queries-and-change-proposals.md)
owns its server-free source/query/proposal routes. G01 fixes the supported semantic
subcommands against the pinned backend. The intended operations are `explain`, `symbols`,
`refs`, `calls`, `implementations`, `macro`, `tests`, `candidates` and `edit`, with explicit
position/range or qualified-symbol subjects. Session `prepare`, `status` and `stop` belong
to this same discoverable family; help/status do not prepare it. D03 `outline`/`show` remain
server-free syntactic reads, not aliases for semantic resolution.

An `explain` response combines available declaration/signature/type presentation, definition,
trait origin and a short source excerpt. A range adds only selected annotations: intermediate
types, captures, adjustments, argument names or relevant exits. Additional references, calls,
tests, docs and edits are next actions, not automatic work in every request. A combined response
aims for approximately 800–1,200 tokens through producer-owned compact views under A03;
generic top-level array budgeting alone cannot establish that result. This is not a new tokenizer,
byte truncation contract or permission to omit required status/context.

The A03 envelope retains provider, selected package/crate-instance/target/features, synchronized
analysis generation, source locations, availability and semantic scope. It distinguishes no
match, incomplete model, preparation required, unsupported capability, canceled/stale analysis
and projected detail. Lists report known shown/omitted counts or an unknown total. Search scope,
completed examination and displayed subset remain separate. A bounded response can require
substantial reference search; presentation limits do not establish cheap execution.

Each semantic producer owns an indivisible result/page view, serialized through A03's
directional typed schema. Continuations bind analysis/source generation and backend context;
refuse stale cursors or serve explicitly retained immutable inputs. Do not recursively trim
tuples/tagged records or equate serializer/schema agreement with semantic adequacy. Preserve
opaque evidence as opaque and keep checked semantic applicability with its actual owner.

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
| [G01](#g01) | complete | 59g | Exact pinned LSP/client mechanism implemented and exercised against the real analyzer |
| [G02](#g02) | complete | 59g | Original B03 native session, real socket-peer authentication, explicit restricted/macro preparation, independent request cancellation and original drain |
| [G03](#g03) | complete | 59g | Selected Cargo model, scalar/UTF-8/UTF-16 conversion, active-file/manifest fences and source-bound lazy candidate identity; exact compiler-unit context remains explicitly unsupported |
| [G04](#g04) | complete | 59g | Native explanation/symbols and selected hints with lazy resolution; unsupported typed HIR fields remain explicit |
| [G05](#g05) | complete | 59g | File-grouped references, one-hop calls, implementations, predicate outcomes and actual proc-macro expansion with invocation fallback |
| [G06](#g06) | complete | 59g | Related-test/completion producers, lazy resolution and actual test-owner enumeration through a produced next action |
| [G07](#g07) | complete | 59g | Source-bound rename, lazy assist, scoped semantic SSR, file-rename and complete supported workspace-effect previews into J02 |
| [G08](#g08) | complete | 59g | Preview-only decision below; no apply command is advertised |
| [G09](#g09) | complete | 59g | Native selected CLI/lifecycle journeys and 678-test host run pass; original diagnostics offer guarded, explicitly current-source semantics without inventing original-source identity |
| [G10](#g10) | complete | 59g | Existing-session native name discovery→guarded public inspection and selected source/refinement/navigation consumers pass; P02 scope-end qualification remains separately owned |
| [G11](#g11) | complete | 59g | Compatible0.0.358 typed/method/reference/Cargo-VFS probes select the A11-published immutable snapshot; SSR non-adoption/revisit recorded below; product acceptance belongs to G12 |
| [G12](#g12) | complete | 59g | Selected immutable helper, original native IDE/context/type/role consumers and authenticated independent-request cancellation/drain tested; P02 scope-end qualification remains separate |

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

**Pinned integration selection — 2026-10-30.** The installed analyzer identifies as
`1.101.0-nightly (c1070d6, 2026-09-28)`. Exact source inspection selects standard definition,
type-definition, hover, signature, inlay/resolve, references, call hierarchy, implementation,
completion/resolve and rename/code-action routes. Selected Rust extensions supply related tests,
runnables, macro expansion, predicate evaluation and semantic SSR. Capture/adjustment annotations
are rendered hints, reference categories are discarded by the LSP handler, and macro expansion
has no exact source map: those typed fields remain unsupported rather than parsed from text.
Native IDE/HIR integration is not selected initially; its existing material-gap trigger remains.

Use `lspf =1.0.3` as the proposed client/stdio integration: its exact published source uses
`gen-lsp-types =0.11.0`, matching the inspected analyzer protocol family, owns request correlation,
abandoned-request cancellation, EOF failures and its child. The host retains original native
containment/settlement. `async-lsp 0.2.4` would require additional cancellation/correlation
machinery; inspected `async-lsp-client 0.2.3` has unsuitable clone-ID/child ownership behavior.
These are source-based choices, not a working-session claim. Qualify the actual client spawn,
initialize, concurrent requests, cancellation and shutdown before dependent consumers.

Further exact-source inspection found that `lspf 1.0.3`'s convenient client initialization
discards `InitializeResult` and its spawn builder supplies neither root URI nor workspace
folders. That convenience route therefore does not satisfy the selected capability/position/
workspace contract. No client dependency has been pinned. Compare a different maintained client
with a narrow adapter over the library's public generic transport and established framing;
retain the actual initialization evidence and original child owner. This qualification replaces
any assumption above that the convenient spawn route alone is sufficient.

The implemented adapter uses the library's public generic transport, `lsp-server =0.10.0`
for native framing and generated standard request types. It supplies the actual process/workspace
root and captures the unmodified actual initialization result before forwarding it. The installed
analyzer negotiates UTF-8; UTF-16 conversion is independently exercised. Native framework request
correlation, abandoned-request cancellation and EOF failures remain library-owned.

The pinned configuration uses `checkOnSave: false`; `check.enable` is not a supported setting.
Build-script override and proc-macro preparation must use the actual pinned configuration and
existing admission route. Preserve analyzer-context ambiguity where LSP cannot choose a crate
instance rather than claiming exact compiler-unit selection. Source leads are the
[pinned analyzer source](https://github.com/rust-lang/rust/tree/c1070d6/src/tools/rust-analyzer),
its `lsp/ext.rs`, `handlers/request.rs` and `config.rs`, and the
[client adoption contract](https://lspf.dev/guides/client-adoption).

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
owners; neither permits writing generated output. Focused navigation may link these views with package
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

**Consumes:** G03, J02's working shared proposal contract and the selected G01 rename/code-action/SSR capabilities. D06 protects
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

**Source-only producer ownership (accepted review RC04,2026-10-30).**
[59j J03](59j-source-queries-and-change-proposals.md#j03) owns structural pattern/capture queries
and previews; it consumes J01/J02, not G03. J02 owns the shared source-bound proposal and diff
contract. Translate semantic source changes into that contract with original context,
source identities, unsupported regions and unresolved snippets. Do not silently downgrade a
semantic request to syntax matching. Tree-sitter grammar, ast-grep matching and typed fragment
qualification live with J03; generated contributor navigation consumes J04/D06 where available.
This relocation removes the former accidental semantic-session barrier without giving syntax
queries semantic authority. No standalone parser daemon or whole-file authored-source
roundtrip through syn/prettyplease is selected.

**Acceptance:** rename changes the intended entity without unrelated names; a scoped SSR
rule excludes a similarly spelled different function. A chosen assist shows all files and
unresolved placeholders. Source change invalidates the proposal. Preview performs no edits,
generation, compiler check or commit.

<a id="g08"></a>
## G08 — Conditional application decision

**Consumes:** J02's working proposal contract, an actual J03 or G07 producer and D06's
protected/generated ownership contract. Syntax application qualification does not consume
G07/G03; semantic application adds its actual semantic producer. This decision is distinct
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

**Decision — preview only, 2026-10-30.** J02 checks exact original sources and effects without
claiming an application transaction. `git apply --check` followed by application leaves a
check/write race; atomic replacements of individual files do not create a multi-file transaction;
advisory locks cannot fence editors that do not participate. A private Git index protects index
publication, not working files. Compensating rollback could overwrite a subsequent unrelated
edit. None establishes the required concurrency boundary for this shared working checkout, so
no apply command is present. Revisit when a repeated selected editing workflow has an available
isolation/cooperative publication mechanism that resolves that boundary.

<a id="g09"></a>
## G09 — Consumer composition and qualification

**Consumes:** selected working G04/G05/G06/G07 producers, B02 original
diagnostic source-generation references and A06 discovery. G08's conditional apply outcome
does not block initial semantic reading. Full H/I capabilities are not prerequisites.

Expose source-linked selected views for O16 directly; no generic packet composer is required. Join an original compiler diagnostic to its matching source generation or expose
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
| Tolerant source parsing | Selected under D03 after the October30 full capability assessment: `ra_ap_syntax` provides useful incomplete-file reads without a semantic session. Preserve errors, source identity and explicit edition; no compiler acceptance claim. |
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

**Implemented — selected semantic delivery, 2026-10-30.** `pse code session prepare` selects
project/features/defaults/platform and explicitly starts or reuses the original B03-owned
service. `status` and `stop` remain passive/existing-control routes. The short private socket
authenticates the actual `SO_PEERCRED` PID against original payload custody before sending a
request. The fixed native entry also checks its original command and immutable selected model.
No arbitrary LSP method, executable, editor attachment or analyzer-produced argv is exposed.

Restricted preparation disables automatic checking, build scripts and proc macros. Explicit
`--build-macros` uses the original immutable `cargo-pse-correctness` command with Cargo JSON,
selected features/platform and native admission; `checkOnSave` and build-script rerun-on-save
stay disabled. Initial native Cargo loading activates the model; after native quiescence the
adapter answers the pinned backend's `workspace/configuration` request with automatic Cargo
reload disabled. It observes the actual configuration response write and completes an ordered
native roundtrip before exposing semantic queries. Readiness retains the pending/failed exchange
separately. Native VFS/Salsa invalidation remains analyzer-owned. Actual backend health/pending
state and captured-context mismatch remain visible. Request-scoped client buffers
close after the last overlapping query, returning disk updates to native watches without scanning
every previously visited file on subsequent queries.

`explain`, `symbols`, `refs`, `calls`, `implementations`, `predicate`, `macro`, `tests`,
`candidates`/`candidate`, `hints`/`hint` and `edit rename|file-rename|assists|assist|ssr` are
working selected consumers. CLI columns are one-based Unicode scalars; provider coordinates
retain both exact UTF-8 byte/source identity and negotiated LSP units. Lazy selections require
original source/generation and an exact discovery-record fingerprint; a bare index is not an
identity. Selected multi-file edits discover affected inputs, synchronize only those inputs and
recompute through the native producer before translating into J02. New/changed affected inputs,
manifest/toolchain changes and stale candidates refuse rather than joining generations. Native
create/rename/delete and snippet intent remain explicit; overwrite/ignore and unrepresented
sequential-edit semantics refuse. There is no source application or second semantic graph.

**Tested — Linux, pinned analyzer/toolchain, zero failure baseline.** Targeted
`scripts/pse test pse-host --lib -E 'test(semantic::) | test(cli::control::)'` exercises the real
analyzer, Unicode/CRLF, same-spelled methods, stale source/configuration, cancellation/reuse,
original native CLI lifecycle, socket spoof refusal, proposal effects/snippets and shared original
owner settlement composition. The original targeted set contained eight semantic tests and three existing
control tests; unselected tests are not qualification. `scripts/pse check pse-host --all-targets`
passed for the implementation closure. These are pure host correctness closures; no Arrow leaf
or scientific execution is selected.

Actual separate CLI journeys exercised source-linked references/calls/tests, lazy hint/completion/
assist resolution, rename/SSR previews, multi-file affected-input recomputation, file-rename
effects and old-source refusal. Pausing the real analyzer, canceling one socket client and resuming
proved request removal plus reuse of the same original service. Passive status/stop also worked
with intentionally broken current host source, restored byte-exactly afterward. A real build
script supplied its cfg through Cargo JSON and real proc-macro children expanded a generated
constant; original native groups were observed drained after stop. An actual checkout-model
related-test result produced a `pse tests -p pse-host --lib -E 'test(=...)'` next action that the
original test owner successfully enumerated. Enumeration is not execution or affected coverage.
The manifest-effect witness changes default features after explicit macro preparation: native
status observes a blocked reload, the query refuses stale context, and the build-script execution
count remains one. The retained native lifecycle test exercises that effect boundary.

**Implemented and Tested — context-freshness correction, Linux, 2026-10-30.** Effective Cargo
workspace manifests/locks and discovered ancestor Cargo/toolchain configuration are captured
before analyzer launch, including a member selected below a virtual workspace root. Native
metadata is confirmed against that capture; fixed configuration watches retain transient
edit/restore and absent-path creation, including in restricted sessions. Explicit
macro preparation runs `check --workspace --all-targets`. A fixed native watch set covers the
metadata-selected package trees and extends once at preparation with declared rerun inputs from
the pinned Cargo's native `build_directory` outputs. Cargo target/intermediate output trees and
VCS metadata are excluded. Any watched input/topology change, overflow or lost watch refuses
status currency, query delivery and preparation reuse until original stop/drain and explicit
preparation; even ordinary Rust edits conservatively invalidate a macro-prepared session.
Preparation never clears intervening source changes merely because the backend is quiescent.
Queries drain native events without source-tree rescans or implicit compilation. Undeclared
external build-script/proc-macro I/O remains unknown; this is a bounded freshness claim, not
exact compiler-input custody. The targeted semantic tests exercise actual member-project root
manifest/lock edits, authored build-script cfg, compiled proc-macro source and Cargo-declared
external inputs, refusal without changed compiler outputs, fresh expansions after explicit
preparation, and original native drain. Focused watch tests cover preparation-time edits,
separate native output directories and overlapping missing/directory rerun inputs.
`scripts/pse test pse-host --lib -E 'test(semantic::)'` passed all thirteen selected semantic
tests against a zero failure baseline; `scripts/pse check pse-host --all-targets` passed for this
correction. Unselected product/scientific scope remains unqualified by these focused checks.

**Outcome and handoff.** The initial `cargo.autoreload: false` choice prevented the pinned
backend from activating its workspace; actual resolution tests caught this. Auto-reload is now
enabled only for explicit initial activation and disabled through the actual pinned reverse-config
exchange afterward. `rebuildOnSave: false` alone does not block a manifest-triggered compiler
invocation: the pinned native workspace switch can queue build-data fetching independently.
Deliberate limits are exact compiler-unit/crate-
instance selection, typed capture/adjustment/reference-category fields, exact macro source maps
and application. Native IDE/HIR remains the existing material-gap trigger, not an automatic
fallback. No performance benefit or scientific correctness is claimed. The coordinator retains
G09's adjacent original-diagnostic/discovery composition, integrated review, assembled acceptance
and scope-end static/manual qualification; this delegated result does not retire the plan or
claim the complete series accepted.

## Planned search extension packages

These packages consume the [query/result foundations](59k-query-and-result-foundations.md)
and supply the [composed investigation plan](59m-composed-rust-investigations.md).
The preceding completion evidence covers G01–G09 only.

<a id="g10"></a>
### G10 — Name-first discovery through the existing analyzer

**Consumes:** K01's agreed source/predicate/answer model for design; working K02 requests,
results and locators for public integration; existing G02/G03 session, selection and source
synchronization. Source-only declaration discovery belongs to M01, not this semantic producer.
A10 publishes changed governing contracts before dependent implementation.

Expose `workspace/symbol` and selected rust-analyzer workspace-symbol extensions through the
existing owned LSP process. Check the installed protocol's actual query/kind/scope support;
retain an honest client-side filter when a requested filter is not native. Resolve a simple or
qualified name into candidates with package/module/kind, available visibility and source location,
then allow inspection/references/calls without reconstructing coordinates. Ordinary written
declarations can still be found without semantic preparation through M01. Do not present
document symbols as exhaustive workspace discovery or confuse acquired rustdoc identities with
live semantic entities. Preserve ambiguous ranking and the actual searched Cargo context.

**Public consumer and acceptance:** `search` finds a name shared by unrelated traits, then
`inspect` selects one declaration and one relevant reference through K02 locators and M01.
Focused native tests distinguish exact/fuzzy candidates, reexports, a dependency item, inactive
cfg and same-file multiple crate contexts. Missing preparation returns preparation-required;
stale generation, unsupported kind/scope and unknown compiler-unit identity stay explicit.
Changing source/configuration invalidates the result before a follow-up. No search request
starts Cargo or another analyzer. Test actual public navigation, not only protocol snapshots.

**Migration/deletion:** add this producer to the common model; move existing symbol/navigation
consumers through M03 and remove their replaced public parser branches, guidance and tests in
the delivering slice. Keep explicit session control and useful distinct semantic investigations.
Delete no read merely because a new workspace-symbol request exists.

**Implemented:** the original semantic service accepts name-first workspace requests through
the single prepared session. Its existing LSP owner synchronizes newly discovered files and
repeats the native query until ranges can be bound to one checked generation and original byte
digests. Portable follow-ups retain original managed-session invocation, source hash and
generation. A changed session/source/configuration refuses. Native container labels do not
prove a full module path or visibility, so unsupported semantic filters refuse; source-only
selection uses actual AST visibility/module/self-receiver metadata. Fuzzy symbol output remains
non-exhaustive even below the client allowance, so zero rows do not prove absence.

**Tested — selected consumers:** the native component filter
`cargo t -p pse-host --lib --features native-hir -E 'test(semantic::native::tests)'`
passes three controls against the zero-failure baseline. They include exact qualified native
module discovery excluding an unrelated same basename, selected dependency/std discovery,
source/manifest staleness and independent cancellation. The actual public namesake fixture
`actual_native_namesakes_refine_by_target_then_inspect_relationship_preview_and_test_selection`
also passes its source-candidate refinement, navigation, relationships and preview controls.
The current public namesake journey additionally exercises ready-session semantic symbol
discovery into a source/session/generation-guarded result and public inspection. Compact native
sections round-trip through their shared DTO with empty handles intentionally omitted; mandatory
omission counts remain required. G10's targeted implementation acceptance is complete. Exact
public command accounting and the separate remaining scope-end/static qualification belong to P02.

<a id="g11"></a>
### G11 — Native IDE/HIR, SSR and snapshot feasibility

**Consumes:** K01's design questions and existing G01/G03 capability/context evidence. This
inquiry can run before K02 implementation, G10 delivery or any compiler-driver decision.

Select representative questions: a resolved method target rather than same spelling; inferred
versus adjusted type; a generic trait constraint; and scoped read/write references. Compare
extending the existing analyzer interface, a narrowly extended upstream server and one embedded
`AnalysisHost`/HIR owner. Inspect an exact compatible RA family as a unit: overview HIR/IDE
0.0.358 and current syntax0.0.360 do not establish shared AST compatibility. Compare complete
Cargo/VFS/build-script/proc-macro loading, change application, snapshots, cancellation, source
mapping and upgrade obligations. `ra_ap_load_cargo` batch loading is not a live update service.
Do not maintain two independently updated semantic worlds for the same project/context.

Qualify `Semantics::type_of_expr`/`type_of_pat`, `TypeInfo` original/adjusted types, method/field
resolution, `could_coerce_to`, owner-aware substitutions and visibility/import maps for the
selected questions. Include implicit operation leads `resolve_bin_expr`, `resolve_index_expr`,
`resolve_try_expr` and `resolve_await_to_poll`; `find_all_refs`, `SearchScope` and
`ReferenceCategory` support scoped role queries where exact public APIs permit. Display strings
are not type equality; shallow impl enumeration does not settle blanket applicability.
Extract plain results in the owning worker rather than sharing `Semantics` across threads.

Investigate SSR separately: `MatchFinder::in_context`, `add_search_pattern(SsrPattern)` and
`matches()` exist, but examined `Match` ranges/nodes/placeholders are private and public
`matched_text()` is insufficient for addressed captures. Check selected-release accessors or a
bounded upstream/source-integrated extension against ast-grep candidates plus HIR enrichment.
Never obtain search identity from identity rewrites or parsed edit output. Ordered rewrite
precedence is distinct from Boolean alternatives.

For an explicit offline/historical question, compare `ra_ap_ide::StaticIndex` or native
`rust-analyzer scip` with the existing warm owner. Occurrences, definitions/body ranges,
monikers and docs may support bounded navigation; an export binds source/configuration and
explicit preparation. It does not provide automatic freshness, rename-proof identity or current
semantics for historical bytes. Snapshot support is conditional, not a required index service.

**Decision and acceptance:** record exact-family/interface evidence, question fit, one selected
provider per supported capability/context, unsupported fields and adoption/no-adoption reasons.
In the authorized feasibility phase use bounded real cases with same-spelled method targets,
adjustments, blanket/generic impls, expansion fallback, stale snapshots and cancellation. An
SSR branch must demonstrate public addressed ranges/captures; text-only output cannot pass.
A snapshot branch must refuse source/configuration mismatch and distinguish live freshness.
Missing access or preparation is an unresolved/unsupported result, not capability absence.
Send only selected governing changes to A11; leave ordinary search and G10 independent.

**Selected realization — 2026-10-31:** the isolated exact0.0.360 HIR/IDE resolution failed because
the registry supplies HIR/IDE0.0.358 while syntax0.0.360 exists. An all0.0.358 native family then
compiled and ran under the pinned rustc1.101.0-nightly (`c1070d693`), after correcting the
fresh-resolution Unicode18 properties0.1.5 versus Unicode17 ident1.0.24 compile-time mismatch
with properties0.1.4. These are failed attempts/corrections, not capability absence. The final
run demonstrates distinct targets for two same-spelled impl methods, a constrained generic
method, four inferred/adjusted type observations, native read/write roles (one each), an
AnalysisHost snapshot and an actual Cargo/VFS-loaded fixture with three resolved methods.
The fixture deliberately disables build scripts/proc macros; live watching and prepared macro
integration are not established by it.

Select an explicitly prepared immutable native-analysis snapshot using that compatible family,
ordinary serialized facts, existing semantic custody/preparation and an optional helper binary.
The native context is separate from live-LSP freshness and never an independently updated second
world. A11/blueprint321 publish this selection before G12. Existing source-only syntax0.0.360
does not cross native AST boundaries. In the examined0.0.358 SSR source, public `matches()` and
`matched_text()` do not expose addressed `Match` ranges/placeholders; no native SSR-search branch
is selected. Existing syntax captures plus selected HIR refinement remain useful. Revisit SSR
when the selected release exposes addressed captures or a separately selected bounded upstream
extension supplies them; rewrite output is not a search substitute. G12 must qualify the actual
product context, source/configuration guards, cancellation and public consumers before closure.

<a id="g12"></a>
### G12 — Conditional native provider implementation

**Consumes:** a positive G11 decision naming supported questions, exact compatible family and
provider ownership; the applicable A11 publication; and working K02 integration contracts.
There is no dependency on I06/I07/A12. If G11 selects no native branch, record that decision and
its precise revisit trigger; do not implement a backend to satisfy the package number.

Implement only selected typed HIR/IDE operations and any selected snapshot/SSR capability.
Use native change/snapshot/Salsa ownership, the existing native process/preparation boundary and
shared query/result views. Reuse loading/watchers by extending the existing owner where selected;
if replacing it with an embedded owner, move all affected G consumers and retire the replaced
world in the same coherent cutover. Keep not-proven, incomplete, unsupported, stale and canceled
distinct. No hover/debug parsing, implicit compiler work or claim of rustc equivalence is allowed.
An optionally selected term-construction mode may use `hir::term_search`, `TermSearchConfig` or
native completion fuel controls; preserve fuel exhaustion, non-exhaustiveness and proposed import
effects. Native diagnostics remain a separately identified overlay on original compiler evidence.

**Public consumer and acceptance:** a selected syntax/name candidate is refined to an actual
entity/type/role, inspected and expanded through M02. Independent cases challenge inferred versus
adjusted types, implicit calls, cfg/crate context, ambiguous/blanket impls and multi-origin macro
mapping. Exact editable authored ranges differ from invocation-only fallback. Actual source,
manifest and macro-input changes invalidate old results; cancellation settles owned work and
leaves surviving queries usable. An adopted snapshot or SSR branch runs its G11 adverse cases
through public `pse`. Missing preparation leaves ordinary source/API reads usable.

**Deletion and limits:** remove the superseded semantic acquisition/database path and consumers
when replaced; retain one semantic world owner per project/context. M03 removes replaced public
read routes with their useful replacements, guidance and tests. No persistent cross-repository
index, second live database or unselected typed field follows from adopting one native operation.
Targeted compile/tests establish only selected branches; the coordinator owns scope-end journeys
and static qualification. No quantitative performance benefit is claimed by this plan.

**Implemented — 2026-10-31:** the optional native-analysis helper now uses
the same prepared-tool custody and semantic-session owner as the live provider, selecting one
immutable Cargo/VFS/HIR world explicitly. The actual loaded crate graph supplies library source
roots, package IDs, effective feature sets, target and original source digests; query-time
package-context reads run no Cargo. Installed standard-library sources join that same world;
explicit preparation checks `rust-src` first and supplies its source override, avoiding the
loader's component-install fallback. Dependency-symbol discovery explicitly searches both native
workspace and library indexes; the LSP route refuses that widened domain when its protocol
cannot establish it. Native facts retain inferred/adjusted types, resolved declarations,
read/write roles and destination visibility/use-path evidence without interpreting display text
as type identity. A fixed authenticated cancellation control awaits actual CPU settlement and
preserves independent query/session ownership. Exact published Salsa0.28.5 and
`ra_ap_ide_db`0.0.358 sources carry a narrow opt-in cancellation-group patch: cloned native IDE
and scoped Rayon handles share a monotonic request signal while preserving each handle's
local suppression/reset state and the original shared inputs. Ordinary Salsa clone behavior
outside that branch is unchanged. Original native IDE methods remain authoritative; the
temporary lower-level replacement operation module was deleted. Archive checksums, patch scope,
licenses and reproduction commands live in each vendor's `pse-patch.md`.

**Tested — selected source/runtime outcome:** the native component command above passes
3/3, covering typed targets/adjustments/roles, actual native visibility/use paths, guarded
transitive feature profiles and cold-symbol cancellation followed by independent/fresh queries.
The cold test observes actual coarse CPU ownership; it does not establish the exact internal
indexing instruction at cancellation or a universal latency bound. Focused library commands
`cargo test --manifest-path vendor/salsa/Cargo.toml --locked --lib cancellation_group_tests`
and `cargo test --manifest-path vendor/salsa/Cargo.toml --locked --test parallel cancellation_group`
pass 2/2 and 1/1 respectively, including barrier-confirmed tracked computations on nested Rayon
clones and recomputation from an independent request.

The actual authenticated native lifecycle command
`cargo t -p pse-host --lib --features native-hir -E 'test(semantic::tests::native_snapshot_authenticated_cancel_settles_one_request_and_preserves_the_session)'`
passes 1/1 against zero failures. Its isolated operational namespace explicitly prepares the
helper, starts and authenticates the original native session, cancels one active request,
observes CPU-settled acknowledgement, retains a successful sibling/current reusable session
and drains the original service. This route uses ordinary raw Cargo because independent
persistent-session creation inside an enclosing `pse test` workload is intentionally refused;
no ownership marker or production refusal is removed. Earlier startup/preparation/timeout
attempts supplied no acceptance credit. Broader API composition and the coordinator's full
P02/hygiene qualification have their separate result at the coordinator; no scientific or quantitative performance
qualification follows from these controls. G12's selected targeted implementation acceptance is
complete; P02 owns the integrated/static result.
