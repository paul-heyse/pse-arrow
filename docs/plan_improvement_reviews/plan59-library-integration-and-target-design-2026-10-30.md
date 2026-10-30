---
title: Plan59 library integration and target design
status: proposed
date: 2026-10-30
doc_role: review
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [architecture, design, libraries, environment, diagnostics, validation]
---

# Plan59 library integration and target design

## Judgment, baseline and limits

**Proposed:** keep the useful native front door, but organize its remaining development around
complete user journeys and shared semantic contracts. Fuller library delegation can remove
coordination and representation work; adding library wrappers alone would retain the hardest
problems. Reconsider implemented foundations as well as scheduled capabilities. Prior effort is
not a reason to preserve a mechanism or an awkward command boundary.

The inspected baseline is `09e878957`, following the paused [Plan59 checkpoint](../plans/59-agent-cli.md#current-state-and-next-step).
Untracked library documents and the local uncompiled provenance draft were preserved. This
review covers A–I, current `pse-host` implementation and supporting consumers materially changed
by those plans. Current architecture owners, especially blueprint §21.7, §23.2 and §24.1,
describe the baseline; ADR-0233/0234 remain proposed records. Recommendations do not amend them.
Existing opportunity identifiers keep their original review-qualified identities and Plan59
disposition owners. The I-identifiers below belong only to this document.

The selected standard is Core3.4, efficiency heuristics1.0, test assessment1.2 and the applicable
Process Simulator1.5 guidance under the repository binding. AP-01–AP-07 and DP-01–DP-24 inform
the judgment; this is not an exhaustive gate verdict or certification of complete alignment.
Scientific validity, numerical acceptance and physical process settlement remain independent
of architectural quality and CLI success. PS-10/PS-12 particularly constrain observation and
presentation; this review does not revisit equations or numerical solver selection.

**Interface-checked:** the identified source paths and enabling library interfaces, at the
specific limits discussed below. **Proposed:** the amendments and expected benefits. No new
runtime implementation, qualification campaign or performance measurement was performed.
The preceding checkpoint records 351 host unit and19 selected CLI passes and zero-warning
Clippy in its named modes; these historical results were not rerun or promoted to review acceptance.

## Integrated target

The target has five cooperating responsibilities, not five mandatory processes or crates:

1. **Intent and staged selection:** the owning parser resolves requested policy and effects;
   deeper preparation, admitted selection and actual captured population remain distinct stages.
2. **Execution custody:** one shared service path for managed foreground/detached work, with
   direct exec adapters retaining their narrow protocol. Passive reads remain direct. Reusable
   analyzer sessions have their own lifetime; canceling a query does not terminate the session.
3. **Evidence and publication:** exact original streams and immutable artifacts retain their
   source, while protected local publication and scientific state retain their existing owners.
4. **Owned query producers:** Git, syntax, semantic analysis, compiler diagnostics and product
   inspection answer their own questions without being forced into one universal knowledge graph.
5. **Presentation and navigation:** concise task-shaped answers, typed next actions and optional
   detail compose those producers. The shell remains usable without a resident CLI daemon.

The largest design gains come from reducing repeated interpretation and unnecessary work:
effect-shaped generation protection, direct invocation lookup, independent stream capture,
query-aware detail selection and a shared source-bound edit proposal. Strong types and schemas
support these boundaries; neither substitutes for the operations that enforce them.

## Recommendations

<a id="I01"></a>
### I01 — Resolve once at each semantic stage

**Alignment improvement; A03/A06, B01/B02, C02/C07, D02, F01, G03, H/I.**
Keep parsed intent, resolved plan, admitted operation and observed execution distinct. Explain
and execution consume the same resolved stage value rather than separately interpreting argv,
environment, feature selection or defaults. Missing preparation appears as a prerequisite, not
as permission for explain to run setup or invent an actual test population. Capture the selected
stage and relevant premises with results; invalidate or refuse when they change before effects.

This completes C07's shared selection direction without introducing a universal operation
registry or eagerly loading every provider. Domain-specific values can share small outer
contracts for identity, effects and availability. Keep actual nextest listing, compiler artifacts
and scientific outcomes with their original producers. A new operation should add its parser,
owned handler/result and discovery association, not switches that independently reinterpret policy.

Compare retaining duplicated explain/execution logic: it is cheaper initially but permits
different selections. Compare a universal fully resolved object: it couples unrelated providers
and makes passive explanations effectful. The staged design avoids both (AP-01–AP-06,
DP-01/05/08/18/24). Close with one selected explain→execute journey, changed-premise refusal,
and an unavailable fixture/provider whose explanation performs no preparation.

<a id="I02"></a>
### I02 — Finish one managed custody path before adding more run kinds

**Missing accepted obligation and alignment improvement; B01/B03, A05, C04, G02.**
The current subgroup, readiness and cooperative-shutdown primitives do not yet form an assembled
managed invocation. Complete the ADR-0234 service supervisor/finalizer journey before copying
foreground/detached lifecycle logic into tests, captures or analysis sessions. Retain original
unit incarnation, opened cgroup identity, enclosing allocation and actual payload subtree.
Close launch authority before recursive termination, drain every independently owned obligation,
and publish command outcome separately from physical settlement.

Use `tokio-graceful-shutdown` for a small hierarchy of long-lived control/capture/cleanup
responsibilities, and Tokio/tokio-util primitives for ordinary work inside them. Do not create
a subsystem per output chunk or small file. Required evidence readers must survive cancellation
of a view or the main control task long enough for their explicit drain policy. `JoinSet` and
`TaskTracker` are alternatives for narrower task populations, not additional registries to stack
on the same work. `TaskTracker::close` allows wait completion; it does not prohibit spawning.
Cancellation tokens signal intent; neither tokens nor task termination prove OS cleanup.

Retain the raw adapter's special argv/exit protocol and direct passive readers. A shared
executing owner does not require every command to become a service. Close with actual
foreground/detached journeys, origin loss, nested escape, failed launch acknowledgement,
independent cleanup failures and original allocation release (AP-03/05/06, DP-19/20/21).

<a id="I03"></a>
### I03 — Let native observation own reconciliation and fan-out

**Library opportunity; B03, F02/F03 and adjacent host observation.**
`manager.rs::observe_inner` constructs uncached proxies and performs multiple serial property
reads. Use typed zbus_systemd list operations for summaries and interface-scoped `GetAll` where
appropriate for details. Preserve the incarnation fence: several interfaces are not one atomic
snapshot. Existing stop/drain code already subscribes to property changes and cgroup events;
extend that owner rather than building a second polling supervisor.

One request/session-scoped reconciliation owner can serve latest-state consumers through
`watch` and optional transition consumers through `broadcast`. Lag, disconnects and manager
replacement require explicit resynchronization. Current state is not lossless event history.
Keep checkpoint polling where the product source has no push facility. There is no requirement
for a new resident host daemon.

Use native sd-notify readiness/stopping/status at the authenticated service boundary. Evaluate
watchdog, socket activation and fdstore only for a concrete lifecycle need. Pinned sd-notify0.5
does not implement a notification barrier. Those mechanisms do
not automatically transfer opened payload custody to a stop-post finalizer. Generated APIs
must be checked against the installed systemd, not merely the generator's release. Close with
one observation race, unavailable interface and lag/resynchronization case. Latency benefits
remain unmeasured (DP-13/14/15, AP-07).

<a id="I04"></a>
### I04 — Make generation protection follow actual retained effects

**Observed amplification and target correction; A05, B03, E01, G02.**
`cli.rs::main` acquires the prepared entry guard before dispatch; `GenerationUse::acquire`
loads the manifest, hashes binaries and durably publishes a use record. Even a passive
source/status entry consequently enters a persistent protection protocol. Abrupt termination
can leave an unresolved claim. The paused worker checkouts illustrate unresolved-claim
consequences, not established attribution of those particular claims to passive readers;
absence of original PIDs is not evidence authorizing deletion.

Separate temporary read protection from durable obligations for children, exported executable
paths or finalizers. A candidate is a process-held shared generation guard, acquired under the
same exclusion protocol as publication/pruning, promoted to a recorded obligation before
retained effects. Qualify descriptor inheritance, promotion and pruning races before selecting
that mechanism. Do not silently clear existing unknown claims or presume an ephemeral lock
proves descendant completion. Keeping pruning disabled remains the safe current behavior.

Distinguish source equivalence checks, prepared-manifest integrity and live-use protection;
they have different inputs and lifetimes. Reuse established immutable integrity only under
an explicit trust/mutation boundary. Do not replace content validity with an mtime heuristic
or hash the whole repository on every brief query. Compare retaining the current durable
protocol: safe conservatism, but unnecessary persistent obligations for no-spawn reads.
Close with killed passive reads, child/path export, source-copy handoff and concurrent pruning
under the selected protocol (AP-04/07, DP-03/09/19; H18/H23/H25).

<a id="I05"></a>
### I05 — Decouple stdout/stderr capture and simplify raw retention

**Observed coupling and library opportunity; B01/B02/B03, F03.**
`managed/streams.rs::State::accept` serializes shared state while writing retained chunks.
A stalled sink can delay both readers despite independent child pipes. Prefer independent
raw stdout/stderr files with per-stream offsets and a compact attributed index only when
observational interleaving is useful. This preserves bytes without base64/JSON wrapping every
chunk in the retained raw representation. JSON transport may still encode non-UTF8 bytes.
Cross-pipe index order is observer order, never an exact producer execution order.

Use native Tokio reads, cancellation-safe framing and owned writer/drain tasks. Required
capture has an explicit sink-failure policy; optional terminal/follow subscribers use bounded
queues and visible loss. No renderer holds the capture lock or blocks a native callback.
Prefer tokio-util codecs where they preserve oversized/truncated raw records; a default codec
that discards evidence is not a conforming replacement. Settle writers independently and retain
each original failure. Migrate B02 extraction through exact stream/offset references; replace
the old format and its mechanism tests together after consumers move.

Compare retaining merged NDJSON as the only log: easy inspection, but encoding, shared I/O and
reconstruction work on every chunk. An index has its own consistency burden; omit it when
stream attribution/offsets alone suffice. Close with a deliberately blocked sink, invalid UTF8,
oversize message, broken optional subscriber and failed final flush (AP-01/07, DP-10/19/20).

<a id="I06"></a>
### I06 — Make exact invocation access independent of unrelated history

**Observed defect and execution-fit improvement; A04/A06, B01/B03.**
`InvocationStore::resolve` calls `history<RawValue>` before matching even a complete UUID.
`history` validates and decodes every metadata record and sorts the collection. An unrelated
malformed record therefore prevents resolving a healthy exact invocation; control also scales
with all retained metadata. This follows from the inspected source, not a newly executed test.

For an exact ID, validate and open only its bound directory/record. For a prefix, enumerate
candidate identities and establish ambiguity before opening the chosen record; never silently
choose a healthy candidate while hiding another identity match. History/latest remain collection
questions with declared ordering, per-record diagnostics and explicit incomplete coverage.
An unreadable record may prevent a trustworthy latest selection; do not label a partial scan
as the global latest. Avoid a second writable history index initially.

Make status summary selection reach acquisition: current `status::inspect` reads Git status,
source freshness, full history and result records before producing a brief view. A cheap view
should request only the facts it can use, with explicit omitted/unexamined sources. Keep a
full inspection option. Close with healthy exact ID plus unrelated corrupt metadata, ambiguous
prefix, partial history and a summary that does not read every terminal result (AP-06/07,
DP-08/10/21; H5/H6/H26).

<a id="I07"></a>
### I07 — Give producers useful detail contracts; keep projection mechanical

**Observed limitation and functional enhancement; A03/A06, D01/D03, F–I.**
`Presentation` budgets top-level payload arrays. `FileOutline` stores entries beneath
`payload.outline.entries`; a single outline can therefore remain large under a small budget.
A batch budget counts files, not their nested declarations. Generic post-serialization
projection cannot deliver the planned compact code-intelligence answer by itself.

Give each producer a small set of meaningful query/detail modes and pageable result units.
For source reads, page selected declarations with enclosing context and explicit continuation;
for code explanations, select declaration/type/relevant excerpt before optional references;
for failures, choose original diagnostics and related spans before optional raw output.
Do not introduce a universal JSON query language or tokenizer. Generic fields/budget remain
mechanical views over producer results and must preserve mandatory status/identity/coverage.

Push selection into acquisition where semantics permit. A bounded reference list may still
require a broad semantic search; report examined scope separately. Full artifact retention is
valuable for original evidence, but should not automatically serialize a huge already-rebuildable
query result merely because the user requested one page. Close with nested outline and batch
cases, explicit omitted detail, unknown totals and a failed continuation (AP-02/07, DP-08/10/21).

Continuations bind source digest/revision and backend context. After an edit, refuse a stale
cursor or serve a deliberately retained immutable source. A derived query is reconstructible
only while its inputs remain available; dirty working bytes and sole original diagnostic
evidence must not disappear under a generic rebuildable-cache policy. Distinguish fresh-query
continuations from exact-original-source continuations in the producer contract.

<a id="I08"></a>
### I08 — Share source-bound proposals across distinct editing mechanisms

**Library opportunity and dependency correction; D03/D06/D08, G03/G07/G08.**
Use `ra_ap_syntax` for lossless/error-tolerant Rust reads and local CST edits; ast-grep with
tree-sitter-rust for structural pattern/capture/replacement operations; syn/quote/prettyplease
for owned generated syntax/fragments; rust-analyzer for resolved rename/assists/semantic SSR.
These are different capabilities, not fallback implementations of the same semantic promise.

G07's current semantic-session prerequisite must not gate server-free structural previews.
Give syntax queries/previews a direct D03 source route, with an explicit authority/mode and
unsupported-semantic refusal. Share a minimal proposal: addressed original sources, exact
edits/file operations, source identity, backend/context and unresolved placeholders. Backend
syntax objects remain local. Incremental nodes and library-local IDs are not durable symbols.

Do not whole-file round-trip authored code through syn/prettyplease: comments and formatting
have different preservation contracts. Native CST fragments or exact untouched ranges retain
them. Query macros as syntax unless expansion/resolution is actually supplied. Pattern captures
prove shape, not binding identity, trait applicability or borrow correctness.

G08 retains a separate apply decision: preflight alone does not prevent concurrent writes,
and a Git private index is not a multi-file filesystem transaction. Preview remains useful
without apply. Close with malformed Rust, edition/macro differences, same-spelled different
entities, Unicode offsets, overlapping replacements and stale/mid-write failures (DP-08/13/15/19).

<a id="I09"></a>
### I09 — Add addressed Git/source navigation without another code database

**Functional enhancement; D01/D03/D08/D09/E01, G/H.**
Extend native gix observation into selected diff/hunk, rename, history, index/blob and worktree
navigation. Compose change→addressed source→enclosing declaration→query/preview without first
checking out the historical content. Add positional show for an enclosing declaration/range,
not only qualified-name selection. Keep repository/object-format, revision, path bytes and
working/index/blob source kind explicit; HEAD is mutable selection, not an immutable revision.

Reuse one source acquisition layer with immutable text/byte coordinates per request. Avoid an
eager persistent symbol graph or parser tree cache until repeated queries justify its invalidation
and thread-affinity cost. For public commit publication, keep D09's qualified private-index/native
Git transaction where gix cannot discharge the full consumed contract. Library substitution
must preserve staged intent, lock behavior, paths, failure boundaries and the explicitly
supported hook/filter policy; qualify or refuse unsupported modes rather than imply blanket
hook execution or clean/smudge compatibility.

This adds value to concurrent-agent reviews while avoiding a repository browser platform.
Close with staged plus unstaged hunks on one file, rename, linked worktree, nonUTF8 paths and
an independent concurrent index change (AP-02/03/07, DP-04/09/15/19).

<a id="I10"></a>
### I10 — Compose investigation answers from original evidence

**Functional enhancement; D01, B02, F02/F05, G09, H06, I01–I04.**
Make D01's packet composition an early usable consumer, not the last output-polish package.
A failure-focused answer should connect the invocation's original diagnostic, selected command
context, exact source excerpts, available API constraints and suggested targeted checks.
A simulation answer should connect owner-published statements, phases/ages, coverage and the
next supported observation. A code-change answer should connect selected hunks, symbols,
candidate tests and a preview. Each section preserves provider scope and availability.

No answer silently runs a check, prepares analysis, probes a library or captures a simulation.
Represent next actions as arguments and declared effects, avoiding shell-string reconstruction.
Source edited since the failure is a mismatch, not a replacement of original evidence.
Missing sections remain visible; one unavailable enrichment does not erase the useful answer.

Keep compiler MIR/layout/codegen and advanced library compatibility queries on-demand, with
the existing lighter-route-first policy. Do not require compiler-private preparation before a
useful failure explanation. Close through an actual agent journey from a native failure to a
selected next action, without a second compilation just to explain it (AP-03/06, DP-18/21).

<a id="I11"></a>
### I11 — Expand production observation through the existing scientific owners

**Functional enhancement and preserved target; F01–F06, B03.**
Make process/unit observations, invocation metadata and scientific observation distinct sections
of one useful answer. Reuse `Statement {kind,evidence,next}` and original source references for
why-state explanations. Preserve hypotheses, limitations, clock/coverage and unknown causal
relationships. Low CPU or an old checkpoint is not a hang diagnosis. Static code navigation
is not an executed dependency or a scientific cancellation permission.

Use existing product checkpoints and retained study history first. Compare any new cross-process
numerical publication against the exact unavailable question, bounded write/retention burden
and existing progress. Keep SurrealDB canonical transitions and long-term scientific records
with their product owner; an object-store artifact does not become that state model.
Observation views detach independently, preserving the run. Optional OTel/Perfetto enrichment
stays optional; no monitoring backend is required for local inspection.

Close with two observers of one run, observer timeout, publication reset/gap, failed science
under a successful read, and missing live numerics. This retains the earlier observability
review's O01–O04/F dispositions rather than renumbering or replacing them (PS-10/12,
AP-04, DP-04/05/07/21).

<a id="I12"></a>
### I12 — Separate artifact I/O from authoritative local publication

**Library opportunity and boundary decision; A03/A04/A05, B01, D02, F05, H/I.**
Keep `object_store` for addressed immutable evidence and bounded/ranged or streaming reads.
Useful additions are suffix/tail access, selected record windows and selected evidence export;
these serve the recurring need to inspect large logs without loading every byte. Use native
`GetOptions`, `GetRange`, stream payloads and validators, preserving byte offsets and encoding.
ETag/version/hash remain distinct; local mutable reads do not gain snapshot isolation.

Do not replace private local publication, inode/UID/no-follow checks or Git transactions with
the generic store trait. Exact installed `object_store0.13.2` local source rejects
`PutMode::Update`; it follows symlinks. Generic conditional-write documentation therefore does
not establish a safe local CAS ledger or multi-file apply mechanism. Current artifact reading
already honestly reports `atomic_snapshot=false` and refuses unsupported native path keys.

Classify retention by responsibility: regenerable preparation, sole retained original diagnostic
evidence, uniquely authored probes, mutable Git intent and scientific persisted data have
different disposal rules. A common resolver can address them without a common deletion policy.
Remote export is a separate selected capability: provider credential builders, multipart uploads,
explicit abort, native retry and optional signing can help when an actual transfer is needed.
Do not enable them or bump the single object_store family merely to match the newer overview.
Close with mid-stream failure, changed validator, unsupported local CAS, tail bounds and retained
authored inputs (AP-01/04, DP-15/19; H24/H25).

<a id="I13"></a>
### I13 — Treat schema, decoding and valid operation construction as one contract

**Alignment improvement; A02/A03/A06, B01/B02, F01, G/H/I adapters.**
Derive wire structure and directional schemas from the owning Serde/Schemars types. Keep
serialize/deserialization views distinct where defaults, skips, flattening or custom conversions
differ. Decode external domain inputs into checked constructors; the output envelope should
be a projection of valid owner results, not the mechanism that independently establishes their
meaning. Public independent outcome fields need owner-level construction rules for valid
combinations; do not create a single enum covering every possible product state.

Concretely, `InvocationDirectory::publish_result` accepts the generic envelope;
`validate_result` checks binding/version/non-running/non-projected status, not terminal child
or reconciliation evidence. Establish an owner-finalization boundary in B01/B03 before that
generic value becomes an authoritative terminal record. This is a missing target enforcement
boundary, not demonstrated false success in the unfinished joined supervisor. Serialize the
owner result into the query envelope; enforce admission/finality where their evidence is owned.

Preserve native typed errors before rendering. Dynamic facts may use deliberate opaque payloads,
but once selection/control depends on a field it needs an owned typed contract. Schema validity,
decode success and domain validity are separate claims. Retain RawValue for opaque protocol
evidence rather than normalization that erases duplicate keys or numeric lexemes. Evaluate
schema customization against current projection logic without inventing a schema framework.

Current `OutputOptions::emit` generates a schema which `presentation::write` generates again.
Normal writes serialize typed payloads to RawValue and reserialize an outer frame. Remove the
duplicate generation; use borrowed typed page/view serialization for owned records. Preserve
private frame/spool preparation where it prevents serialization failure emitting a partial
record. For genuinely opaque upstream data, retain RawValue's lexical fidelity.

Use native Schemars `Schema::pointer` for URI-fragment references rather than stripping `#`
and delegating to Value::pointer, which omits the native percent decoding. Prefer derived
view types and narrow native transforms over a general interpreter for `properties/type/$ref`.
The existing tuple-prefix schema test relaxes the schema after truncating a tuple: agreement
with the transformed schema does not establish semantic fidelity of an indivisible record.
Do not repair nested budgeting by recursively cutting arbitrary tagged records or tuples.
Serde_json's nonfinite-to-null behavior likewise requires domain-level finite/absence policy
before numerical facts are emitted; JSON serialization alone cannot preserve that distinction.

Use actual clap value sources and captured environment/configuration origins for provenance,
not equality with defaults. Redact values while retaining origin and dependency identity.
Close with explicit-default versus implicit-default, unknown schema/protocol version, malformed
input, custom serializer/schema agreement and an invalid domain combination reaching the
proper constructor (AP-04/05, DP-01/02/03/21/24).

<a id="I14"></a>
### I14 — Qualify vertical journeys and retire replaced machinery continuously

**Execution and assurance improvement; all packages.**
Select ready producer→consumer slices rather than implementing all generic foundations before
an assembled journey. First finish managed invocation/capture/settlement; then actual test
selection/reconciliation and independent existing-work control. Bring syntax query/preview
and base product observation forward on their narrower prerequisites. Neither requires full
semantic or compiler-private tooling. Add API and semantic enrichment to working packets as
their producers become ready; retire duplicated callers/tests/fixtures when actually replaced.

Challenge shared helpers with independent expected behavior: schema acceptance alone does not
establish correct source selection; the same parser cannot supply both actual and expected
symbol meaning; a successful child does not establish verdict completeness or drain. Test
the shared domain constructor locally and the real OS/provider boundary where its guarantee
matters. Reuse immutable preparation without erasing freshness and feature-absence cases.
Avoid a general test framework, mandatory benchmark or another qualification ledger.

Close each slice with the actual CLI journey plus its distinguishing failure case, preserving
the existing force-validation, capacity and scope-end static-check routes. Full scientific
qualification remains separately owned. This is an adoption sequence, not a new completion
claim (AP-06/07, DP-16/22/23 and test-assessment §§1–8).

<a id="I15"></a>
### I15 — Resolve compatible package roots together

**Observed amplification and alignment opportunity; C01/C02/C05/C07, H03.**
`selection::plan` creates one group per package; `Plan::resolve_with` resolves them serially.
Each correctness admission reconstructs Cargo metadata/unit-graph work. A workspace intent can
therefore repeat preparation by package count even though its compatible roots share context.
Represent groups as sets of compatible package/target roots, reuse metadata and admit the
selected closure together where semantics permit. Preserve host/target and opt-in feature
policies; never union feature-absence cases into an all-features approximation.

This requires adapting single-package root validation and discovery identity mapping, not
merely putting existing scalar calls behind `join_all`. Keep official nextest filtering/listing
as the selected population authority and preserve force-validation for the actual closure.
Compare retaining per-package groups for incompatible contexts: that remains correct and useful.
Close with mixed compatible/incompatible roots, duplicate test names and feature-negative controls.
Structural repeated work is source-supported; elapsed-time improvement is unmeasured
(AP-03/07, DP-09/10/20; H8/H12/H22).

<a id="I16"></a>
### I16 — Follow generated hits to the actual producer

**Functional enhancement; D03/D06, G05/G07, D01.**
The existing code generator's `SourceRelationship` records declaration key/kind, generator
path/symbol, output path and source-role/identity relationships in
`pse-codegen::codegen::provenance`. Compose those records with D06's regeneration routing so a
generated hit offers its known emitter/declaration and regeneration action. Do not reconstruct
ownership from matching symbol spellings, fabricate missing registry source locations, or
turn generated output into an editing authority.

This completes an important change→owner→source→preview workflow while avoiding another
provenance index. Extend producers only for a demonstrated missing relationship. The generated
output map and contributor lineage answer different questions; one must not overwrite the other.
Close with a generated diagnostic/hit followed to its emitter, an unknown authored location and
an explicit regeneration next action without editing output (DP-01/04/07/21, AP-03/06).

## Interactions, priorities and proposed rule impacts

The highest consequence corrections are I02/I04 (ownership and retention), I05 (capture
isolation), I06 (existing-work access) and I13 (enforced domain/wire contracts). Useful delivery
order differs: settle I01's stage boundaries and I13's required constructors while implementing
I02's first journey; I06 and server-free I07/I08 can proceed independently. I03 improves the
same observation owner; it is not a prerequisite to all CLI functionality. I09/I10 compose
working producers; I11 has an independent base product-reader route. I12 supports artifacts
without becoming the control store. I14 governs migration and acceptance across these slices.
I15 should accompany C07's first actual multi-package consumer; I16 can accompany the first
generated-source investigation. Neither requires full semantic-session readiness.

Retain all substantive user outcomes unless consolidation provides a better journey. Merge
duplicated status/brief/packet assembly behind producer selection, not by deleting useful
question-specific commands merely to reduce verb count. Defer remote artifact transfer, automatic
idle eviction, a resident CLI broker, global source graph, generic transaction framework and
compiler-private driver until their specific capability/benefit is established. The current
lighter-route and preview-only outcomes remain legitimate, useful deliveries.

Proposed authority impacts for subsequent plan creation:

| Reference | Potential change | Route |
| --- | --- | --- |
| RC01 | Effect-shaped generation protection instead of durable records for every passive entry; no retrospective retirement authority | A05/A07, ADR-0233/0234 and blueprint §21.7/§24.1 as affected |
| RC02 | Independent raw streams/optional offset index replace the merged retained-chunk format | A03/A04, B01/B02 consumers and existing commit/protocol decision route where applicable |
| RC03 | Query-specific/pageable response contracts and shared edit proposals may intentionally change immature wire shapes | A03/A06, D03/G07/G08, versioned schema migration; no compatibility shim required by this review |
| RC04 | Syntax previews do not consume semantic-session readiness | G07 dependency/target amendment and affected lifecycle binding |

These proposals preserve scientific authority, clean-room boundaries, explicit effects and
native resource placement. No new crate, mandatory service, MCP installation, database policy
or dependency change is selected by publication. A design decision due under the binding still
needs its own route; this plan-improvement review does not replace it.

## Research coverage and evidence

All twelve originally supplied references were read end to end across the runtime researcher,
editing researcher and root. The independent architecture reviewer additionally read both
Serde/Schemars references in full. Capability breadth was considered before narrowing to amendments.
The following source map preserves consequential leads and exclusions; library names are not
claims that every feature should be integrated.

| Library coverage | Useful mechanisms and assessed boundary |
| --- | --- |
| zbus/zbus_systemd | Typed proxies, jobs, signals, properties, service ownership and FD interfaces: I02/I03. Network/login/time service families are not part of this CLI's selected responsibilities. |
| Tokio | Task/channel/process/I/O/time and runtime diagnostics: I02/I03/I05. No new runtime metrics service or per-command runtime tuning selected. |
| tokio-util | Cancellation, task tracking, codecs, I/O adapters and keyed expiry: I02/I05. DelayQueue/expiry is not mandatory analyzer eviction; framing must retain required raw evidence. |
| tokio-rayon | Shared-pool async bridge, ordering/panic/cancellation behavior: I07/I08. Keep worker-local syntax objects, return owned records; dropping the awaiter does not stop CPU work. |
| tokio-graceful-shutdown | Hierarchy, failure actions, detached/partial/ordered shutdown: I02. Avoid duplicate task registries and signal handlers; no OS completion claim. |
| sd-notify | Readiness/status/stopping, watchdogs, socket activation/fdstore: I03. Barrier support is a pinned-library gap. Additional activation/FD protocols remain conditional rather than finalizer shortcuts. |
| object_store | Conditions/ranges/streaming, multipart, listing/copy/delete, builders/auth/transport, attributes/extensions/signing, adapters/registries and conformance: I12. No global storage migration, custom backend or destructive provider conformance campaign. |
| gix | Discovery, objects, revisions, diff/rename/merge, index/status/worktrees, transport, features/security: I09. Remote fetch/clone or full commit plumbing is not adopted simply because available. |
| ra_ap_syntax | Lossless CST/typed access, recovery/editions, traversal, editing and incremental reparse: I07/I08. No resolution or compiler correctness inferred. |
| ast-grep-core | Matching strictness/composition/captures, replacements, traversal and extensibility: I08. Structural matches are not resolved SSR; no custom multilingual platform selected. |
| syn/quote/prettyplease | Typed parsing/visitors/folds/generics/token generation and formatting: I08. Owned generation/fragments, not lossless authored-file roundtrips. |
| tree-sitter-rust | Grammar/scanner/macros, queries/captures, recovery/incrementality and grammar tooling: I08. Prefer the selected pattern backend's grammar; no separate always-running parser service or grammar fork. |
| Schemars | Complete overview read by the independent reviewer: directionality, derive/customization, references/transforms, dialect/settings and schema testing inform I07/I13. Schema generation is not instance or domain validation. |
| Serde/serde_json | Complete overview read by the independent reviewer: representations, visitors/borrowing, attributes/conversions, RawValue/streaming/errors and number/duplicate behavior inform I05/I07/I13. Preserve opaque evidence; avoid a generic JSON domain model. |

Primary mechanism references include [Tokio task tracking](https://docs.rs/tokio-util/0.7.19/tokio_util/task/struct.TaskTracker.html),
[Tokio channels](https://docs.rs/tokio/1.53.3/tokio/sync/index.html),
[graceful subsystem management](https://docs.rs/tokio-graceful-shutdown/0.20.0/tokio_graceful_shutdown/),
[systemd manager interfaces](https://www.freedesktop.org/software/systemd/man/latest/org.freedesktop.systemd1.html),
[sd-notify](https://docs.rs/sd-notify/0.5.0/sd_notify/),
[object_store local source](https://docs.rs/object_store/0.13.2/src/object_store/local.rs.html),
[object conditions](https://docs.rs/object_store/0.13.2/object_store/struct.GetOptions.html),
[gix](https://docs.rs/gix/0.90.0/gix/),
[Rust syntax](https://docs.rs/ra_ap_syntax/0.0.360/ra_ap_syntax/),
[ast-grep](https://docs.rs/ast-grep-core/0.51.0/ast_grep_core/),
[tree-sitter Rust](https://github.com/tree-sitter/tree-sitter-rust),
and [syn](https://docs.rs/syn/3.0.5/syn/).
Additional exact-pin references are [Schemars](https://docs.rs/schemars/1.2.2/schemars/),
[Serde conversion attributes](https://serde.rs/container-attrs.html#try_from),
and [serde_json RawValue](https://docs.rs/serde_json/1.0.151/serde_json/value/struct.RawValue.html).
Context7 was the first library lookup, followed by exact installed sources for consequential
version/backend details. The object-store overview targets0.14.2; the repository holds0.13.2
to preserve the DataFusion family. Tokio/util overviews also describe newer versions than
the selected pins. An overview's available feature is not an exact-pin integration guarantee.
The added Schemars/serde_json overviews target1.2.3/1.0.154 versus repository1.2.2/1.0.151;
the relevant mechanisms above were checked against installed pinned sources, without a bump.

Independent architecture assessment covered the full selected standard and challenged the
integrated draft. Its corrections are incorporated: staged resolution does not imply eager
preparation; managed execution sharing excludes passive readers; historical claims are not
causally attributed without evidence; Git hook/filter guarantees stay scoped; and source
continuations retain their input premises. This is independent advice, not formal acceptance.

## Publication and follow-up

The review is a planning input. Recommendation adoption, plan/dependency edits, ADR changes,
production implementation and qualification remain separate. No existing package completion
was changed here. Broad audit accounts and historical reviews were not refreshed. Validation
of this publication establishes document consistency only, not the runtime recommendations.

**Tested (documents only):** `target/debug/xtask lifecycle validate` validated1,698 scope items
with zero errors against baseline zero. `just turn-end` and `git diff --check` passed.
No product tests, native provider preparation, lifecycle probe or benchmark ran for this review.
