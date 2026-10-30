---
title: "Plan59d: Knowledge and workflow verbs"
status: active
date: 2026-10-29
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, design, libraries]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md]
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

Each verb is a host subcommand. Where the owner of the knowledge is xtask, the verb is
**federated**: its schema comes from the generated xtask export ([59a A06](59a-cli-foundation.md#a06)),
and dispatching it runs xtask through the correctness adapter. Prebuilt xtask is reused when
current.

| Verb | Owner | Behaviour |
| --- | --- | --- |
| `pse changes [--base REF]` | host | Changed paths grouped by owning crate; generated and protected paths with their regeneration verb (from the map below); affected tests as a ready `pse test --changed`; applicable `.claude/rules/*` by path scope; content-hash hunk IDs for `pse commit`. |
| `pse brief [--paths …] [--libraries …]` | host | A delegation packet: HEAD and dirty state, active plan pointers, applicable rule files with content hashes, exact pins, features and overview paths for the named libraries, the evidence-label vocabulary, and document conventions. |
| `pse dep <crate>` | host | Exact locked version, enabled features, workspace users, `[workspace.metadata.pse.pins]` reason, locked source path and `docs/libraries` overview. |
| `pse api <crate> <path>`, `pse api <crate> --search W` | host | Item signature, docs, feature gates and source location, from rustdoc JSON generated once per (crate, version, features). Cached in `<store>/api/`. Re-exports into another crate also generate that crate's JSON. `format_version` is checked first; a mismatch is one diagnostic naming the `rustdoc-types` bump. |
| `pse probe new\|run\|rm <name> --deps …` | host | Scratch crate under `<store>/probes/` with a copied `rust-toolchain.toml`, a `Cargo.lock` seeded from the workspace lock, and the workspace's feature selections. It runs through the ordinary placement path. Probes are throwaway. |
| `pse outline <file>`, `pse show <file>::<item>` | host | Server-free syntactic extraction with line numbers: items, impls and tests via `syn` with span locations; headings for Markdown. |
| `pse owner <path>` | host | Generator plus narrowest regeneration verb for generated paths; section ID plus route for architecture documents; supersession for accepted ADRs; crate and owning plan for code. |
| `pse docs check [--changed\|<files…>]` | xtask (federated) | Pinned `lychee --offline --include-fragments` over the files, plus `metadata.rs` front-matter validation, `#section-N-M` link resolution against `section_owners`, review `O`-anchor checks and README index presence. Reported as O02 diagnostics. |
| `pse ref <ID>` | xtask (federated) | ADR number → title, status, decision and supersession (`adr::inspect`); `§N.M` → section file and anchor (`section_owners`); plan packet, review `O` IDs and DP/AP IDs → file, heading and lead paragraph. |
| `pse new review\|plan\|probe <slug>` | xtask (federated) | Pre-decided placement and name, front matter, standard version from `standard.toml`, HEAD as baseline, evidence labels and a README index stub. `pse new plan` moves `just plan`'s bash numbering into xtask. `pse new adr` routes to `xtask adr new`. |
| `pse commit --paths … [--hunk ID…] -m … [--trailer …]` | host | Builds the commit from a private `GIT_INDEX_FILE` (HEAD plus exactly the named paths or hunks). Advances HEAD with a compare-and-swap (`git update-ref HEAD <new> <old>`), then refreshes only those entries in the shared index. Hunk IDs are content hashes and a changed hunk refuses. Files committed whole are formatted with the pinned formatters before commit; the formatted content is written back only if the file is unchanged since it was read. In hunk mode formatting is reported only. A touched generator input adds a `next` hint to run its scoped `codegen-check`; it is not a gate. Trailers are applied with `git interpret-trailers`. Never pushes. |

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

- **Federate rather than port documentation verbs.** xtask already owns metadata, section
  anchors, the ADR index and lifecycle. Porting them into the host would duplicate those
  interpreters (DP-01). The federated schema keeps them in the `pse` catalog and help.
- **lychee for links.** A pinned, maintained link checker with fragment support (DP-13) does the
  work. Only the publish-time section anchors need the xtask-side map.
- **`rustdoc-types` for API lookup.** It reads the exact locked version with unified features,
  which docs.rs cannot. The existing `rustdoc-mcp` server is trialled for Claude in D02; if it
  serves, `pse api` reduces to producing the JSON it reads.
- **A private index for commits.** It removes the shared-index race, which was the reason only the
  coordinator committed (R5).

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

<a id="d01"></a>
### D01 — Orientation and delegation briefs

**Consumes:** [59a A03–A04](59a-cli-foundation.md), and the D06 map for generated-path flags.
Until the map exists, use the guard's current prefixes.

**Delivers:** `pse changes` and `pse brief` as described above.

**Acceptance:**
- A tree with one Rust change, one generated-path change and one document change yields three
  groups: the generated path flagged with its verb, and the document flagged with its rule files.
- `pse brief --libraries clap,tokio` lists the exact pins, features and overview paths.

<a id="d02"></a>
### D02 — Dependency facts, API lookup and probes

**Consumes:** [59a A04](59a-cli-foundation.md#a04).

**Delivers, in order:**
1. **The trial.** A `rustdoc-mcp` entry in the repository `.mcp.json` for Claude. The trial
   checks that it builds through the checkout's pinned toolchain (not `+nightly`) and answers item
   lookups at locked versions. The outcome is recorded here.
2. **`pse dep` and `pse probe`.**
3. **`pse api`.** If the trial serves, `pse api` only produces the locked-version JSON the server
   reads. Otherwise it is the full lookup described above.
4. **A pins reason.** A `[workspace.metadata.pse.pins]` reason on `rustdoc-types` tying it to the
   toolchain date.

**Trial outcome (2026-10-29): the server serves only partly; `pse api` is built in full.**
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

So `pse api` owns generation itself: pinned-toolchain JSON per (crate, version, features) into
`<store>/api/`, multi-version addressing, feature gates and the `format_version` check.
`.mcp.json` stays empty.

**Acceptance:**
- `pse api tokio process::Command::kill_on_drop` prints the 1.53.3 signature and docs.
- A re-exported item is found through its re-export.
- A probe depending on `serde_json` compiles under the pinned toolchain with the workspace lock
  unchanged.

<a id="d03"></a>
### D03 — Structural reading

**Consumes:** [59a A03](59a-cli-foundation.md#a03).

**Delivers:** `pse outline` and `pse show` over `syn` (full, span locations) for Rust and over
headings for Markdown, with the outline logic shared with `just lib-outline`.

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
  anchor checks and README index presence.

**Acceptance:**
- A document with one broken relative link, one bad heading fragment and one bad
  `#section-N-M` link yields three diagnostics with file and line.
- A clean new review passes.

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

**Consumes:** [59a A06](59a-cli-foundation.md#a06) (the catalog walker).

**Delivers:**
- `just codegen` runs `cargo run -p pse-host --bin pse -- commands --markdown`, so xtask never
  has to depend on `pse-host`.
- The output is a Markdown reference of every non-hidden `pse` verb in `docs/generated/`, with
  `codegen-check` drift.
- It is linked from the documentation site and the environment guide.

**Acceptance:** the reference renders every verb's usage, arguments, possible values and
examples, and `codegen-check` flags drift after a deliberate flag change.

<a id="d08"></a>
### D08 — Shared-tree commits

**Consumes:** [59a A01](59a-cli-foundation.md#a01) (R5 landed), D01 (hunk IDs) and D06 (map).

**Delivers:** `pse commit` as described above.

**Acceptance:**
- **Private index.** In a tree where another file is staged in the shared index, `pse commit
  --paths a.rs` commits only `a.rs` and leaves the other staged entry untouched.
- **Concurrent HEAD.** A HEAD that moved during the commit is refused by the compare-and-swap.
- **Hunk mode.** A hunk whose content changed after listing is refused.
- **Formatting.** An unformatted whole file is formatted before commit, unless the file changed
  after it was read.
- **Drift hint.** A touched generator input yields the scoped `codegen-check` hint in `next`.
- **Trailers.** Trailers appear exactly as passed.

## Verification and handoff

Targeted tests run on `pse-host` and xtask (the `documentation` and `codegen` tests). Real runs:
- `pse docs check` on this series;
- `pse ref` on this series' citations;
- one `pse commit` in a scratch repository.

[59e E03](59e-environment-and-adoption.md#e03) adds each verb to the AGENTS.md task table and the
skills that use it, in the same change as the verb.
