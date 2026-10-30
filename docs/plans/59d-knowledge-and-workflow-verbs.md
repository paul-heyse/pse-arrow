---
title: "Plan59d: Knowledge and workflow verbs"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, design, libraries]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md, docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md]
---

# Knowledge and workflow verbs

Part of [Plan59](59-agent-cli.md).

The current target incorporates the [CLI/test/runtime review](../functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md)
O01/O03. The coordinator owns its dispositions and confirmed rule decisions; historical
implementation evidence below retains its original scope. The selected simplification/testing
wave includes the necessary generated-source and affected-selection slice. This plan adds the verbs that remove an agent's lookup and
bookkeeping work:
- focused changed-path and source facts;
- exact dependency facts and API lookup;
- scratch probes and structural reading;
- owner routing, with the edit guard informing instead of blocking for documents;
- fast documentation checks;
- citation resolution and document scaffolds;
- shared-tree commits.

Sources: [O06](../functional_reviews/rust-agent-cli-2026-10-29.md#O06) (generated reference),
[O07](../functional_reviews/rust-agent-cli-2026-10-29.md#O07),
[O08](../functional_reviews/rust-agent-cli-2026-10-29.md#O08),
[O14](../functional_reviews/rust-agent-cli-2026-10-29.md#O14),
[O15](../functional_reviews/rust-agent-cli-2026-10-29.md#O15),
[O16](../functional_reviews/rust-agent-cli-2026-10-29.md#O16),
[O17](../functional_reviews/rust-agent-cli-2026-10-29.md#O17),
[O18](../functional_reviews/rust-agent-cli-2026-10-29.md#O18).

The amendment incorporates [new O07–O08](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O07),
[I09](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I09),
[I10](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I10) and
[I11](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I11).
The target is **Proposed**; no stopped/reverted implementation is reinstated. D02's recorded
trial remains evidence of that trial only.

Rule basis:
- [R4](59-agent-cli.md#rule-changes): documents inform instead of block.
- [R5](59-agent-cli.md#rule-changes): any agent commits its own paths.

Excluded by operator direction: the rust-analyzer MCP for Codex and the `rustdoc-mcp` trial for
Codex.

## Baseline

The following facts come from a code map at `68e075493`.

**Generated outputs.**
- Generated outputs are declared in three agreeing places: the guard's prefixes in
  `agent_hooks.rs`, the `.claude/settings.json` deny list, and AGENTS.md prime directive 2.
- The codegen side has a partial table, `pse-codegen` `Language::roots()` covering the Rust,
  Markdown and Surreal roots. Other roots are added in code: the xtask private store, gRPC wire,
  physical quantities, document schemas and `pse-ipopt-sys/src/bindings.rs`.
- No table maps an output path to its `xtask codegen --only` target.
- `affected` keeps its own generator path list.

**Edit guard.**
- `agent_hooks::protected()` always blocks `.git`, `target`, `build`, `external` and the
  generated paths.
- It blocks `docs/authoritative_design/*` and non-proposed `docs/adr/NNNN-*.md` unless
  `PSE_DESIGN_EDIT=1` or an untracked `.design-edit` file is present.
- A block prints `BLOCKED: <path>: <reason>` and exits 2.
- There is no advisory channel.
- The `.claude/settings.json` deny list covers only the unconditional set.
- In the review window, Codex set `PSE_DESIGN_EDIT=1` inline 23 times, which never reaches the
  guard, and so wrote protected documents through the shell.

**Documentation owners in xtask.**
- `documentation/metadata.rs`: `read`, `validate_fields`.
- `site.rs`: page discovery, staging, and `#section-N-M` anchors injected at publish time from
  `adr::section_owners`.
- `adr.rs`: `inspect` for the lint report, the index, `next_number` and `section_owners`.
- `lifecycle.rs`: `Query`, `execute`, `snapshot`.
- The plan scaffold is a bash recipe (`just plan`); ADR scaffolding is `xtask adr new`.
- `lychee` 0.24.2 is installed but unpinned and runs only in the manually dispatched docs
  workflow (`--offline --include-fragments`).

**Dependency facts.** `cargo_metadata` (a host dependency) gives exact locked versions and
manifest paths. The pinned nightly emits rustdoc JSON format 61, matching `rustdoc-types` 0.61.0.
The research probe generated JSON for a locked dependency with
`cargo rustdoc -p <crate>@<ver> --lib -Z unstable-options --output-format json`.

**Reading.** `syn` 3.0.5 and `proc-macro2` with `span-locations` are pinned in the workspace.
`just lib-outline` outlines Markdown headings.

**Commits.** AGENTS.md currently says the coordinator owns the shared index, that agents stage
only owned paths or hunks, and that formatting happens at `just turn-end`. Several agents share
one checkout.

## Target

Each verb is a host subcommand. Where the owner of the knowledge is xtask, the verb delegates
to its authoritative parser through [59a A06](59a-cli-foundation.md#a06). Owner-generated
discovery/help is associated with the executable/schema version; it cannot reconstruct
arbitrary typed parsers, validators or dynamic semantics. The owner validates before effects.
Preparation is explicit; help/catalogue reads never build an unprepared owner. Execution uses
a current, immutable captured xtask selected through A05 rather than a mutable Cargo output.

| Verb | Owner | Behaviour |
| --- | --- | --- |
| `pse changes [--base REF]` | host | Changed paths grouped by owning crate; generated and protected paths with their regeneration verb (from the map below); affected tests as a ready `pse test --changed`; applicable `.claude/rules/*` by path scope; source-qualified selected diff facts. Generic briefs are excluded and commit integration is deferred. |
| `pse dep <crate>` | host | Exact locked version, enabled features, workspace users, `[workspace.metadata.pse.pins]` reason, locked source path and `docs/libraries` overview. |
| `pse api <crate> <path>`, `pse api <crate> --search W` | host | Item signature, docs, known availability conditions and source location (unknown ordinary-build applicability remains explicit), from rustdoc JSON keyed by exact package/version and source identity, resolved features, target/cfg, pinned toolchain and rustdoc format. Cached in `<store>/api/`. Re-exports generate the addressed crate's JSON in that same context. Check `format_version` before decoding; report unsupported format with a remedy. |
| `pse probe new\|run\|rm <name> --deps …` | host | Scratch crate under `<store>/probes/`, copied toolchain pin, seeded workspace lock and explicit selected features. A separate target/build context isolates artifacts; report actual scratch resolution and its source/target/toolchain identity. The workspace lock is unchanged; a copied lock does not freeze every modified scratch manifest. Runs use ordinary placement and disposal refuses live users. |
| `pse outline <file>`, `pse show <file>::<item>` | host | Server-free syntactic extraction with line numbers: items, impls and tests via `syn` with span locations; headings for Markdown. |
| `pse owner <path>` | host | Generator plus narrowest regeneration verb for generated paths; section ID plus route for architecture documents; supersession for accepted ADRs; crate and owning plan for code. |
| `pse docs check [--changed\|<files…>]` | xtask (delegated) | Pinned `lychee --offline --include-fragments` over named/changed files, plus metadata, section/O-anchor and README index checks. Typed diagnostics distinguish changed-file issues from external/historical issues. No automatic historical cleanup or unrelated check expansion. |

**Generated-outputs map (one authority).**
- `just codegen` emits a generated JSON map. For each `--only` target it records:
  - the target's generator *inputs* (generator source, registry and declaration paths);
  - its *output* path patterns, from `Language::roots()` and the code-added roots;
  - its regeneration command.
- The map is read at runtime by:
  - the edit guard (blocking generated paths);
  - `pse owner`;
  - `pse changes`;
  - `affected` ([59c C03](59c-tests-and-fixtures.md#c03)).
- The guard's inline generated prefixes and `affected`'s generator path list are deleted in
  favour of it.
- The `.claude/settings.json` deny entries remain as harness-level protection with the same
  coverage.

**Guard behaviour after R4:**
- **Blocked, exit 2:** generated paths, `.git`, `target`, `build` and `external`. The message
  names the owner and regeneration verb from the map.
- **Map missing or unreadable:** the guard fails closed. It blocks any path under a `generated`
  directory and `crates/pse-ipopt-sys/src/bindings.rs`, and says the map needs `just codegen`.
- **Allowed, exit 0, with context:** `docs/authoritative_design/*` and accepted ADRs. Context is
  returned as `hookSpecificOutput.additionalContext` JSON, for example "authoritative design:
  record a `blueprint.md` revision row" or "accepted ADR: supersede with `just adr-supersede`".
- **Retired:** `.design-edit`, `PSE_DESIGN_EDIT`, `design_edit_authorized`, and the related
  AGENTS.md, CLAUDE.md and `.claude/rules/decisions.md` wording.
- `adr-lint`'s immutability check remains the build-level guarantee for accepted ADRs.

### Decisions with reasons

- **Delegate rather than port documentation verbs.** xtask already owns metadata, section
  anchors, the ADR index and lifecycle. Porting them into the host would duplicate those
  interpreters (DP-01). Its owner-generated catalogue supplies discovery, while its own parser
  remains authoritative. D07 publishes that discovery without becoming an execution grammar.
- **lychee for links.** A pinned, maintained link checker with fragment support (DP-13) does the
  work. Only the publish-time section anchors need the xtask-side map.
- **`rustdoc-types` for API lookup.** It reads the exact locked version with unified features,
  which docs.rs cannot establish for this checkout. The D02 Claude trial rejected the existing
  `rustdoc-mcp` server under this pin; full lookup and context-keyed JSON generation remain
  planned in `pse api`.
- **A private index for tree construction.** It isolates proposed staging, but HEAD publication
  and shared-index refresh are separate transactions. D09 must settle their interaction before
  D08 adoption; a private index alone does not preserve concurrent same-path staged intent.
- **Delegated product inspection.** [59f F01](59f-simulation-observability-and-diagnostics.md#f01)
  owns the prepared product executable/schema boundary and causal correlation. D's focused views and
  catalogues reference its exact invocation/scientific locators without importing model/runtime
  or Arrow interpretation into the host, or becoming another mutable status ledger.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [D01](#d01) | planned | 59d | Focused `pse changes`; generic briefing/packet composition removed |
| [D02](#d02) | in-progress | 59d | `pse dep` provides source-qualified locked facts, metadata-context features, workspace edges, pin reasons and overview candidates; `pse probe` and full context-keyed `pse api` remain; rejected server trial is concluded |
| [D03](#d03) | complete | 59d | Native lossless outline/show, exact source selections and shared lib-outline route integrated; batch CPU execution and source/schema/actual CLI tests cover malformed/Unicode/ambiguous/CommonMark reads |
| [D04](#d04) | planned | 59d | Federated `pse docs check` with pinned lychee |
| [D05](#d05) | deferred | 59d | Citation/scaffold wrappers deferred pending demonstrated workflow need; existing native routes remain |
| [D06](#d06) | in-progress | 59d | Aggregate-generated actual output map, original contributor routing, `pse owner` generated reads and advisory design/ADR guard integrated; document/source owner federation and adjacent affected consumers remain |
| [D07](#d07) | planned | 59d | Generated Markdown CLI reference under `docs/generated/` with drift check |
| [D08](#d08) | deferred | 59d | Operator-confirmed deferral; D09 investigation retained, no commit CLI implementation required |
| [D09](#d09) | complete | 59d | Native Git transaction strategy selected with 11 passing disposable-repository probes; D08 runtime acceptance remains outstanding |

<a id="d01"></a>
### D01 — Focused changed-path facts

**Consumes:** A03/A04 output and source identity, A06's existing gix observation owner,
D06 only for generated-input/output flags. No generic briefing or handoff product is required.

**Delivers:** `pse changes` only where it adds useful grouping, generator ownership and an
explicit affected-test action beyond native Git. Preserve HEAD/index/working/blob identities,
raw paths, selected diff context and direct-base versus merge-base semantics. Use Gitoxide
revision/tree/pathspec facilities rather than a custom porcelain or ignore parser. Rename
inference remains distinct from changed-path facts; attribution of uncommitted lines is unknown.
J01 can consume addressed source without a checkout or generic packet assembly.

Original failures, API constraints, scientific statements and source explanations remain bounded
views owned by B/F/G/H/I. Their typed next actions carry argv/effects and original context;
missing enrichment or changed source is explicit. No reading operation compiles or prepares.
Remove generic `pse brief`, delegation-packet and cross-producer assembly requirements from
producers and guidance. Retain existing native Git/file tools with distinct useful roles.

**Acceptance:** changed Rust, generated and document paths have correct groups and owner routes;
staged/unstaged, rename and historical selections preserve actual source kind. An original
failure can lead directly to source and a runnable selected action without building a brief.
No content-hash hunk identity is introduced solely for deferred D08.

Removal disposition: the current host CLI, parser catalog and source-hook callers contain no
generic brief/handoff composer or packet injection mechanism to retain or replace. Existing
`status --brief`, focused test/check explanations, parser discovery and environment activation
remain independently useful. The B05 session owner and direct G/H/I source/result links carry
their own meaning without a packet prerequisite; E02 remains superseded and D05/D08 deferred.

<a id="d02"></a>
### D02 — Dependency facts, API lookup and probes

**Consumes:** [59a A04](59a-cli-foundation.md#a04) and A05's immutable selected-artifact/build
identity contract for prepared tools. Probe builds use their own target directory, never the
main checkout's Cargo target.

**Delivers, in order:**
1. **The trial (already concluded).** The Claude-only `rustdoc-mcp` trial checked pinned
   toolchain behavior and locked-version item lookup. Preserve its outcome below; do not
   reinstate the rejected server or repeat its configuration as uncompleted implementation.
2. **`pse dep` and `pse probe`.**
3. **`pse api`.** The full lookup and exact-context JSON generation described above, following
   the concluded trial's rejection of the server route.
4. **A pins reason.** A `[workspace.metadata.pse.pins]` reason on `rustdoc-types` tying it to the
   toolchain date.

**Trial outcome (2026-10-29): the server serves only partly; full `pse api` remains planned.**
`rustdoc-mcp` 0.10.0 over stdio JSON-RPC resolved `tokio::process::Command::kill_on_drop` at
1.53.3 and the `clap::Command::arg` re-export at `clap_builder` 4.6.7, from format-61 JSON. It
cannot be used here, for these reasons:
- **Floating nightly.** It rebuilds missing JSON with a hard-coded `rustup run nightly cargo doc`,
  which ignores `rust-toolchain.toml`. Pre-generated pinned JSON does not survive: the first
  dependency the server builds itself makes Cargo wipe `target/doc`.
- **Shared target.** It writes into the shared `target/doc`.
- **One version per name.** It keys crates by name only, and 62 crate names in the lock have
  more than one version.
- **No feature gates.** Its output carries no feature-gate information.
- **Network fallback.** It can fall back to docs.rs.

So the proposed `pse api` owns generation itself: pinned-toolchain JSON keyed by exact source,
package/version, resolved features, target/cfg and rustdoc format into `<store>/api/`,
multi-version addressing, known availability conditions and the `format_version` check.
Documentation context and `cfg(doc)` do not prove ordinary-build availability; unknown gates
remain unknown and consequential applicability uses the selected probe/check.
This trial leaves `.mcp.json` empty; a future optional product adapter is a separate F06 inquiry.

**Acceptance:**
- `pse api tokio process::Command::kill_on_drop` prints the 1.53.3 signature and docs.
- A re-exported item is found through its re-export.
- A probe depending on `serde_json` compiles under the pinned toolchain with the workspace lock
  unchanged.
- Equal package/version with different source, target/cfg, feature context or toolchain cannot
  reuse the wrong API cache. Probe failure reports its actual scratch resolution, with isolated
  artifacts and no workspace lock or build-output mutation.

<a id="d03"></a>
### D03 — Structural reading

**Consumes:** [59a A03](59a-cli-foundation.md#a03).

**Delivers:** `pse outline` and `pse show` over pinned `ra_ap_syntax` for Rust and over
CommonMark headings for Markdown, with the outline logic shared with `just lib-outline`.
The October30 capability assessment selects lossless, error-tolerant typed Rust ASTs instead
of strict `syn` file parsing: an incomplete function or unrelated syntax error must not erase
the useful declarations elsewhere in the file. Preserve original UTF-8 byte ranges, source
digest, explicit parsing edition, enclosing declaration paths and parse diagnostics. Unknown
or duplicate syntactic paths remain explicit; syntax presence does not establish active cfg,
expanded macro declarations or resolved impl identity. `show` slices the original source,
including associated documentation/attributes, rather than regenerating tokens.
This is syntactic extraction, not semantic symbol resolution; absent or ambiguous items yield
an actionable diagnostic rather than a guessed item or implicit language-server startup.

Implementation references: [syntax capabilities](../libraries/rust_analyzers/ra_ap_syntax.md), published
[0.0.360 source](https://docs.rs/crate/ra_ap_syntax/0.0.360/source/src/lib.rs) and
[typed AST](https://docs.rs/ra_ap_syntax/0.0.360/ra_ap_syntax/ast/). Local constructors,
precedence helpers and `SyntaxEditor` can serve selected syntax-edit proposals under J02/J03;
incremental reparsing is useful only when the consumer already retains a source snapshot.
Do not start a persistent analyzer or implement a separate parser cache for one-file reads.

**Extension ownership:** [59j J01](59j-source-queries-and-change-proposals.md#j01) owns the
review I07/I09 extensions: positional addressed working/index/blob reads, operation-owned
pages and source-bound continuations. D03's completion records only the implemented reader
basis; it does not establish those new outcomes. J02/J03 own shared proposals and structural
previews, G07 semantic production and G08 conditional application. Deferred D08 Git publication
is not a dependency of those capabilities.

Multiple outline operands use one coarse `tokio-rayon` submission and the existing shared
Rayon pool's indexed parallel iteration. Parsing/extraction remain together inside workers;
thread-affine AST handles do not cross the async boundary. Every input position retains its
own read/failed/cancelled outcome. `BatchTask` retains its worker handle across a cancelled
borrowed wait; cancellation skips not-yet-started files, while actual completion waits for
all submitted work to return. No additional pool or host CPU cap is selected. CommonMark's
native metadata-block options exclude YAML/TOML frontmatter while preserving source offsets.

**Acceptance:**
- `pse show crates/pse-host/src/manager.rs::HostManager::observe` prints exactly that method with
  line numbers.
- An outline of a Markdown plan lists its headings with line ranges.
- A broken function body still permits reading a neighboring declaration; parse errors and
  the selected edition remain visible. Unicode ranges slice the exact original bytes, nested
  impl/module paths disambiguate where possible, and duplicate paths refuse `show`.

<a id="d04"></a>
### D04 — Fast documentation checks

**Consumes:** [59a A06](59a-cli-foundation.md#a06) (federated schema).

**Delivers:**
- `lychee` pinned in `.config/diagnostic-tools.toml` and installed through the existing
  `diagnostic-tools install` path.
- `xtask docs check [--changed|<files…>]`, federated as `pse docs check`. It runs lychee offline
  with fragments, `metadata.rs` validation, a section-anchor map from `section_owners`, `O`-ID
  anchor checks and README index presence. Named/changed files bound the diagnostic scope;
  external or historical issues are classified without rewriting unrelated records.

**Acceptance:**
- A document with one broken relative link, one bad heading fragment and one bad
  `#section-N-M` link yields three diagnostics with file and line.
- A clean new review passes.
- An unrelated historical broken link does not trigger cleanup or a broader campaign; changed
  links to external/historical evidence report that category and actionable source context.

<a id="d05"></a>
### D05 — Citations and scaffolds (deferred)

The CLI/runtime review O01 removes these convenience wrappers from required delivery. Existing
files, `just plan`, native ADR commands and skills remain available; none is retired for a
replacement that has not shipped. Revisit only when repeated actual citation/scaffold work
reveals a concrete failure or burden that these existing routes do not address. Record the
consumer and compare native improvements before scheduling federated `pse ref`/`pse new`.
This deferral supplies no prerequisite to source intelligence or other CLI commands.

<a id="d06"></a>
### D06 — Owner routing and the guard change

**Consumes:** [59a A01](59a-cli-foundation.md#a01) (R4 landed) and [59a A06](59a-cli-foundation.md#a06).

**Delivers:**
- The generated-outputs map emitted by `just codegen`, with drift check.
- `pse owner`.
- Reuse declared `SourceRelationship` contributor identities with the output/regeneration map;
  J04 consumes this routing for generated-source drill-through. Missing source locations stay
  unknown; matching names never establish lineage. No parallel provenance index is introduced.
- The guard rewritten to read the map, block generated and VCS/build paths with the owner and
  verb, and allow documents with `additionalContext`.

**Deletes:**
- `design_edit_authorized`, `DESIGN_EDIT_MARKER` and the `PSE_DESIGN_EDIT` handling.
- The guard's inline generated prefixes.
- `affected`'s generator path list, if C03 has moved it.
- The AGENTS.md "Off-limits" authorization paragraph, the CLAUDE.md mention and the
  `.claude/rules/decisions.md` wording about the hook blocking ADR edits, all rewritten as
  informed routes.
- The guard tests that exercised the authorization file.

**Acceptance:**
- **Generated edit.** An edit to a generated file is blocked with its regeneration command.
- **Design edit.** An edit to `docs/authoritative_design/sections/*.md` is allowed and returns
  route context.
- **Accepted ADR.** An edit to an accepted ADR is allowed with the supersession route, and
  `just adr-lint` still flags it at build level.
- **Wiring.** `xtask agent-config-check` accepts the unchanged hook commands.

<a id="d07"></a>
### D07 — Generated CLI reference

**Consumes:** [59a A06](59a-cli-foundation.md#a06)'s parser-owned catalogue/discovery contract
and explicit schema-publication route, including associated owner executable/schema versions.

**Delivers:**
- `just codegen` runs `cargo run -p pse-host --bin pse -- commands --markdown`, so xtask never
  has to depend on `pse-host`.
- The output is a Markdown reference of every non-hidden `pse` verb in `docs/generated/`, with
  `codegen-check` drift.
- It is linked from the documentation site and the environment guide.

**Acceptance:** the reference renders every verb's usage, arguments, possible values and
examples, and `codegen-check` flags drift after a deliberate flag change. Scoped discovery keeps
candidate source/freshness/completeness; missing metadata is unknown rather than an empty valid
domain. Direct and delegated parser rejection agrees; stale discovery cannot dispatch silently
under different semantics. Generating this reference is development publication, never a side
effect of help/status.

<a id="d08"></a>
### D08 — Shared-tree commits (deferred)

The operator explicitly deferred `pse commit` on 2026-10-30 under review O01. D09 remains a
completed bounded investigation, not an obligation to build the transaction. Revisit when a
concrete shared-index publication need warrants implementing and maintaining its guarantee.
Ordinary Git does not offer equivalent conditional publication/shared-index reconciliation.
Preserve path/hunk ownership and staging cautions; do not advertise the deferred CLI as usable.

If selected later, consume D09's investigated strategy, recheck its supported Git hook/filter/
attribute premises, and qualify concurrent staging plus post-publication partial failure before
adoption. No current consumer requires D08, and G08 filesystem application is a separate decision.
A09 owns any required revision to earlier R5/commit-guidance promises.

<a id="d09"></a>
### D09 — Git publication and index reconciliation investigation

The investigation below is completed historical evidence. Its original proposal, criteria and
candidate strategy are retained for a concrete future D08 need; they do not schedule D08 or
block current consumers. Ordinary Git still lacks the investigated transaction's guarantee.

**Consumes:** D08's proposed ownership boundary, the existing shared-checkout workflow and
[I10](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I10).

**Produces:** a bounded decision selecting a concrete Git transaction/reconciliation strategy
before D08 implementation. Compare private-index preparation, conditional HEAD publication and
shared-index locking/conflict checks using actual Git behavior in disposable repositories.
Identify the snapshot/conflict scope needed to preserve unrelated and same-path staged content,
working edits, hunk selection and rename/delete intent. Avoid holding a publication lock across
formatting or unrelated work where private preparation and checked publication suffice.

**Evidence and failure closure:** counterexamples must include moved HEAD; staging on an
unrelated and on the same path during preparation; concurrent file edits; deleted/renamed paths;
formatter failure; and failure after HEAD publication but before safe index reconciliation.
Distinguish refused-before-publication from committed-with-unreconciled-index. Evidence must
inspect surviving staged/dirty bytes as well as the resulting commit tree.

**Exit decision:** record the selected algorithm, its conflict/refusal and post-publication
recovery contract and supporting focused test results here, then amend D08 to consume it.
If no sound strategy is established, leave D08 planned/blocked on that decision and preserve
the existing manual commit route. No claim that Git offers a cross-HEAD/index atomic operation
or that a private index alone resolves concurrency is made by this proposal.

#### D09 decision — private preparation, native staged-intent merge and checked publication

**Interface-checked:** Git's `update-ref <ref> <new> <old>` supplies expected-old ref
publication, and `merge-tree --write-tree` supplies native content/rename merge without
reading or writing the worktree/index. **Tested:** the retained research source
`docs/functional_reviews/probes/plan59-git-publication-2026-10-30.py` ran on Git 2.43.0 through
`scripts/pse-env -- uv run --no-sync python` with eleven disposable-repository cases,
zero failures against baseline zero. This establishes the investigated algorithm over those
cases, not an implemented `pse commit`, crash durability qualification or universal Git-format
support. D08 is now deferred; the following investigated strategy is retained as historical decision evidence, not scheduled implementation.

The D09 investigation selected this candidate strategy for any future D08 adoption:

1. Capture the selected attached branch, expected HEAD, selected source/hunk identities and
   working bytes. Build the proposed tree from that HEAD in a private index, applying only
   selected snapshots. Run pinned formatters on private whole-file snapshots; hunk mode only
   reports formatting. A formatter failure publishes neither ref nor index. Shared working
   files are never rewritten by this path; later/concurrent edits remain ordinary dirty work.
2. Snapshot the actual shared index into another private index and obtain its staged tree.
   Create a private synthetic commit with the captured HEAD as parent, then merge that staged
   intent with the proposed commit using native `merge-tree --write-tree`. This transplants
   staged changes onto the new baseline, including clean same-path nonoverlapping hunks and
   rename/delete intent. Unrelated staged content stays intact. A conflict or unavailable
   native merge refuses before publication, preserving exact shared-index and working bytes;
   no bespoke content merge or automatic conflict resolution is introduced.
3. Prepare the reconciled index privately. Acquire the actual per-worktree `index.lock`
   exclusively only for publication, verify that the shared index still equals the observed
   snapshot, and reject observed changes to the selected branch/source/hunk premises. An
   index changed during private preparation is a retryable refusal, not permission to replace
   the newer staging. Reject unmerged or unqualified index forms before effects.
4. Persist the operation's protected intent/artifact references before publication: branch,
   expected old HEAD, proposed commit, captured index identity and reconciled tree. Publish
   the captured branch with expected-old `update-ref`, not an unqualified `update-ref HEAD`
   that could target a newly checked-out branch. A failed compare-and-swap leaves shared index
   and worktree untouched. Index-lock ownership protects against ordinary Git index writers;
   it is not a lock obeyed by arbitrary working-file editors.
5. After ref publication, install the prepared index through the owned lockfile's atomic
   replacement, retaining original failures and the actual commit identity. If installation,
   recording or ownership checks fail after publication, report
   `committed-with-unreconciled-index`; preserve the current index and private recovery
   artifacts and never retry the commit. Process death can leave uncertain phase/lock evidence:
   reconcile original ownership and actual ref/index state explicitly, not by deleting an
   arbitrary lock or assuming a missing terminal record means no commit.

For recovery, only an authenticated operation with the expected branch/commit and unchanged
captured index may install its recorded reconciled tree under a new owned index lock. Changed
HEAD/index or unknown ownership requires inspection/manual recovery; never reset staging to
the commit tree or run a second publication automatically. Concurrent raw ref manipulation,
writers ignoring Git's index-lock protocol and an atomic multi-file filesystem snapshot are
outside the guarantee. Source rechecks detect observed races; a subsequent edit is preserved
because publication never writes working files.

The probe covers unrelated staging, clean same-path staged hunk transplantation, conflicting
same-path refusal, a staging race during preparation, HEAD movement while holding the index
lock, a branch switch, concurrent dirty edits, rename/delete staging, formatter failure/private
formatting, ordinary Git lock refusal and injected failure after ref publication. It inspects
resulting commit trees, staged bytes, working bytes and distinct outcomes. D08 must implement
this in Rust and qualify its actual durable intent/recovery, path/mode/symlink handling,
unsupported index forms, hunk selection and selected formatter integration. The research
probe is not a Python production controller or a compatibility path.

Native contracts: [update-ref](https://git-scm.com/docs/git-update-ref),
[merge-tree](https://git-scm.com/docs/git-merge-tree),
[Git lockfile protocol](https://github.com/git/git/blob/master/lockfile.h).

## Verification and handoff

### Focused API and failure investigation

H extends D02 acquisition/cache/probe authority without another generator or rejected MCP
server. Exact source/features/tool context remains part of API evidence; hosted JSON, doctest
extraction and local JSON are distinct. B/F/G/H/I expose their own source-linked bounded views
and explicit next actions. No D01 packet composer, base brief or complete enrichment population
is required. Missing detail cannot erase useful original evidence or silently launch a probe.

D03 retains cheap syntax reading; D04 owns scoped Markdown/rustdoc checks; D06 supplies actual
generator ownership; D07 derives parser/discovery reference content. D05/D08 are deferred and
are excluded from current acceptance. Original diagnostic → source/API constraint → selected
next action is the consumer journey, including source mismatch and absent preparation.
Targeted host/xtask tests and real supported command journeys accompany implementation; generic
briefs, citation wrappers and scratch commit adoption are not series completion conditions.



[59e E03](59e-environment-and-adoption.md#e03) adds each verb to the AGENTS.md task table and the
skills that use it, in the same change as the verb.
