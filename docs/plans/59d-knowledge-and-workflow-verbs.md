---
title: "Plan59d: Knowledge and workflow verbs"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, design, libraries]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md]
---

# Knowledge and workflow verbs

Part of [Plan59](59-agent-cli.md). This plan adds the verbs that remove an agent's lookup and
bookkeeping work:
- orientation and delegation briefs;
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
| `pse changes [--base REF]` | host | Changed paths grouped by owning crate; generated and protected paths with their regeneration verb (from the map below); affected tests as a ready `pse test --changed`; applicable `.claude/rules/*` by path scope; content-hash hunk IDs for `pse commit`. |
| `pse brief [--paths …] [--libraries …]` | host | A delegation packet: HEAD and dirty state, active plan pointers, applicable rule files with content hashes, exact pins, features and overview paths for the named libraries, the evidence-label vocabulary, and document conventions. |
| `pse dep <crate>` | host | Exact locked version, enabled features, workspace users, `[workspace.metadata.pse.pins]` reason, locked source path and `docs/libraries` overview. |
| `pse api <crate> <path>`, `pse api <crate> --search W` | host | Item signature, docs, known availability conditions and source location (unknown ordinary-build applicability remains explicit), from rustdoc JSON keyed by exact package/version and source identity, resolved features, target/cfg, pinned toolchain and rustdoc format. Cached in `<store>/api/`. Re-exports generate the addressed crate's JSON in that same context. Check `format_version` before decoding; report unsupported format with a remedy. |
| `pse probe new\|run\|rm <name> --deps …` | host | Scratch crate under `<store>/probes/`, copied toolchain pin, seeded workspace lock and explicit selected features. A separate target/build context isolates artifacts; report actual scratch resolution and its source/target/toolchain identity. The workspace lock is unchanged; a copied lock does not freeze every modified scratch manifest. Runs use ordinary placement and disposal refuses live users. |
| `pse outline <file>`, `pse show <file>::<item>` | host | Server-free syntactic extraction with line numbers: items, impls and tests via `syn` with span locations; headings for Markdown. |
| `pse owner <path>` | host | Generator plus narrowest regeneration verb for generated paths; section ID plus route for architecture documents; supersession for accepted ADRs; crate and owning plan for code. |
| `pse docs check [--changed\|<files…>]` | xtask (delegated) | Pinned `lychee --offline --include-fragments` over named/changed files, plus metadata, section/O-anchor and README index checks. Typed diagnostics distinguish changed-file issues from external/historical issues. No automatic historical cleanup or unrelated check expansion. |
| `pse ref <ID>` | xtask (federated) | ADR number → title, status, decision and supersession (`adr::inspect`); `§N.M` → section file and anchor (`section_owners`); plan packet, review `O` IDs and DP/AP IDs → file, heading and lead paragraph. |
| `pse new review\|plan\|probe <slug>` | xtask (federated) | Pre-decided placement and name, front matter, standard version from `standard.toml`, HEAD as baseline, evidence labels and a README index stub. `pse new plan` moves `just plan`'s bash numbering into xtask. `pse new adr` routes to `xtask adr new`. |
| `pse commit --paths … [--hunk ID…] -m … [--trailer …]` | host | Proposed private-index tree construction (HEAD plus named paths/hunks), conditional HEAD publication and concurrency-safe shared-index reconciliation under the strategy decided by D09. Content-hash hunk IDs refuse stale selection. Whole-file formatting uses pinned tools and preserves concurrent content edits; hunk mode only reports formatting. A touched generator adds its scoped drift-check hint, not a gate. Trailers use `git interpret-trailers`. Report whether publication occurred, including committed-with-unreconciled-index. Never automatically retry publication or push. |

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
  owns the prepared product executable/schema boundary and causal correlation. D's briefs and
  catalogues reference its exact invocation/scientific locators without importing model/runtime
  or Arrow interpretation into the host, or becoming another mutable status ledger.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [D01](#d01) | planned | 59d | `pse changes` and `pse brief` |
| [D02](#d02) | planned | 59d | Trial done (server unusable under the pin; see D02); `pse dep`, `pse probe`, and the full `pse api` with rustdoc JSON cache remain |
| [D03](#d03) | planned | 59d | `pse outline` and `pse show` |
| [D04](#d04) | planned | 59d | Federated `pse docs check` with pinned lychee |
| [D05](#d05) | planned | 59d | Federated `pse ref` and `pse new`; `just plan` bash scaffold retired |
| [D06](#d06) | planned | 59d | Generated-outputs map; `pse owner`; guard informs for documents; `.design-edit`/`PSE_DESIGN_EDIT` retired |
| [D07](#d07) | planned | 59d | Generated Markdown CLI reference under `docs/generated/` with drift check |
| [D08](#d08) | planned | 59d | `pse commit` with private index, hunk IDs, whole-file formatting, scoped drift check and trailers |
| [D09](#d09) | planned | 59d | Bounded Git transaction investigation and exit decision required before D08 implementation/adoption |

<a id="d01"></a>
### D01 — Orientation and delegation briefs

**Consumes:** [59a A03–A04](59a-cli-foundation.md), and the D06 map for generated-path flags.
Until the map exists, use the guard's current prefixes.

**Delivers:** `pse changes` and `pse brief` as described above. Briefs link authoritative plan
state and immutable B invocation evidence, selected failures and pending authored intent. Once
F01 correlation exists they carry its exact product locators; that is incremental integration,
not a prerequisite for the initial source/dependency brief.

**Acceptance:**
- A tree with one Rust change, one generated-path change and one document change yields three
  groups: the generated path flagged with its verb, and the document flagged with its rule files.
- `pse brief --libraries clap,tokio` lists the exact pins, features and overview paths.

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

**Delivers:** `pse outline` and `pse show` over `syn` (full, span locations) for Rust and over
headings for Markdown, with the outline logic shared with `just lib-outline`.
This is syntactic extraction, not semantic symbol resolution; absent or ambiguous items yield
an actionable diagnostic rather than a guessed item or implicit language-server startup.

**Acceptance:**
- `pse show crates/pse-host/src/manager.rs::HostManager::observe` prints exactly that method with
  line numbers.
- An outline of a Markdown plan lists its headings with line ranges.

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
### D05 — Citations and scaffolds

**Consumes:** [59a A06](59a-cli-foundation.md#a06).

**Delivers:**
- `xtask ref <ID>` and `xtask new review|plan|probe <slug>`, federated as `pse ref` and `pse new`.
- Plan numbering moves from the `just plan` recipe into xtask, including the never-reuse count
  over retained and historical plans.
- `pse new adr` routes to `xtask adr new`.

**Deletes:** the `just plan` bash body; the recipe becomes a thin call or is retired.

**Acceptance:**
- `pse ref ADR-0233` gives title, status and decision.
- `pse ref §24.1` gives the section file and anchor.
- `pse new review agent-x` creates a correctly named file with front matter, standard version,
  baseline and a README stub.
- `pse new plan x` picks the next never-used number.

<a id="d06"></a>
### D06 — Owner routing and the guard change

**Consumes:** [59a A01](59a-cli-foundation.md#a01) (R4 landed) and [59a A06](59a-cli-foundation.md#a06).

**Delivers:**
- The generated-outputs map emitted by `just codegen`, with drift check.
- `pse owner`.
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
### D08 — Shared-tree commits

**Consumes:** [59a A01](59a-cli-foundation.md#a01) (R5 landed), D01 (hunk IDs), D06 (map), and
D09's decided transaction strategy with counterexample evidence. D09 gates implementation and
adoption of the commit path; it does not gate unrelated D verbs.

**Delivers:** `pse commit` as described above.

**Acceptance:**
- **Private index.** In a tree where another file is staged in the shared index, `pse commit
  --paths a.rs` commits only `a.rs` and leaves the other staged entry untouched.
- **Same-path staging.** A concurrent stage of different content on the same path survives
  publication/reconciliation, including deleted/renamed entries and partial hunk intent.
- **Concurrent HEAD.** A HEAD that moved during the commit is refused by the compare-and-swap.
- **Hunk mode.** A hunk whose content changed after listing is refused.
- **Formatting.** An unformatted whole file is formatted before commit, unless the file changed
  after it was read. Format the privately captured commit snapshot; never overwrite a shared
  working file merely to match the formatted commit. Report remaining worktree differences.
- **Drift hint.** A touched generator input yields the scoped `codegen-check` hint in `next`.
- **Trailers.** Trailers appear exactly as passed.
- **Publication then failure.** If HEAD advanced and shared-index reconciliation fails, report
  committed-with-unreconciled-index with the commit identity and recovery action. Preserve the
  concurrent index and never retry the commit. Formatter failure or concurrent working-tree
  edits preserve dirty work and accurately report whether a commit was created.

<a id="d09"></a>
### D09 — Git publication and index reconciliation investigation

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
support. D08 remains implementation work.

Select this concrete D08 strategy:

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

### API enrichment and investigation packets

H extends **D02** acquisition/cache/probe authority without another generator or rejected MCP
server. Cache identity includes extraction flags/documentation conditions; API JSON, doctest
extraction and hosted JSON have distinct schema/provenance. **D01** owns O16 packet composition:
selected source/contract, consumers/tests, original diagnostics and API evidence with budgets
and progressive next actions. G09/H06 add producers incrementally; base briefs never wait on
their consumers. No automatic whole-workspace enrichment or second status/knowledge ledger.

**D03** retains cheap syntactic reading when semantics are unprepared. G01 may justify tolerant
parsing for incomplete edits; no gratuitous parser migration. **D04** gains H05's explicit scoped
rustdoc checks alongside Markdown checks. **D06** supplies generated-path ownership to macros
and semantic proposals. **D07** derives the new command reference from actual parser/discovery.

Acceptance adds a packet following original diagnostics without recompiling, examples with
setup/mode without automatic execution and explicit source-generation mismatches. MCP/editor/
Codex configuration exclusions remain; CLI stdio LSP is a distinct selected capability.

Targeted tests run on `pse-host` and xtask (the `documentation` and `codegen` tests). Real runs:
- `pse docs check` on this series;
- `pse ref` on this series' citations;
- one `pse commit` in a scratch repository.

[59e E03](59e-environment-and-adoption.md#e03) adds each verb to the AGENTS.md task table and the
skills that use it, in the same change as the verb.
