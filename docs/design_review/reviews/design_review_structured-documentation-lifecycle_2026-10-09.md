---
title: Structured documentation and evidence lifecycle
date: 2026-10-09
tier: design
purpose: target
standard: Core 3.4; Heuristics 1.0
profile: none (process-simulator not applicable)
baseline: 4c24721e691187e1a5b28398b29722fbde671da8
decision: revise
---

# Structured documentation and evidence lifecycle

*This rewrite replaces a rejected first version. That version recommended authored element
markers, closed status vocabularies, a family schema and validating readers. The maintainer
rejected that direction because it adds agent burden without clearly removing any.*

## Summary

**Decision: Revise.** Behavioral and semantic adequacy fails G1, G2, G6 and G7. Architectural
fitness fails G9: all seven foundations are violated for the content-management
responsibilities in scope.

The authored content is sound: reviews and plans keep a usable, flexible format without
linting. What fails is the mechanics around it:

- agents restate in configuration what documents already say;
- they read whole plans to find where an update belongs;
- they cannot trust evidence without re-answering it.

The recommended target is small:

1. **Delete the external overlay and the publication lists.** The overlay is the
   `lifecycle.toml` defaults, exceptions, table selectors, regex status profiles and
   relationships, with the `document_lifecycle` commands. The lists are `current_work`,
   `assets` and `asset_bundles`. The publisher derives both facts from plan status and links.
2. **Plans get an optional, invisible `<!-- kind: X -->` heading marker and one query.** The
   query returns the sections to update as numbered source lines, beside the skill paragraphs
   that say what to consider. Those paragraphs are quoted, never copied. Existing plans work
   unchanged.
3. **One evidence runner** records what each probe actually used, and reports whether any prior
   answer is current, stale (and why) or uncaptured.
4. **Optionally, and last,** a link-repairing retire command.

Shared policy restated across instruction files is consolidated by deletion, and reviews stay as
they are. Decision governance, producer-owned disposal, concurrent plans with one status owner
each, and publication and search are preserved.

The target is *Proposed*. Its probes ran on copies and in scratch, so they establish
feasibility, not implementation.

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | How reference content is captured and managed, and the agent effort spent on its mechanics. Content covered: design reviews, plans (coordinators, companions, packets, dispositions, checkpoints, outcomes, handoffs), ADRs and the register, architecture sections, dev guides, review/plan evidence, process skills, roles and instructions as content, and `build/` outputs that documents cite |
| Excluded | Library skills; the product and scientific substance of Plans 28 and 33 (corpus samples only); source bodies, `.venv*` and database contents; what the process skills ask agents to reason about |
| Standard | Core 3.4 with Heuristics 1.0 and template 3.4, through the [pse-arrow binding](../design_principles/binding/pse-arrow.md). The process-simulator profile does not apply: the subject changes no scientific model, solver or execution contract. Preserving scientific evidence remains a target obligation (§5.4) |
| Tier / purpose | Design / target |
| Reviewer | Independent design reviewer (delegated), with coordinator-commissioned evidence |
| Decisions | Behavioral adequacy: not adequate. Architectural fitness: not fit. Overall: **Revise** (§11) |
| Proposed disposition owner | A new plan, with Plan 32 closed on its representation-independent work (§10) |

**Functional target.** Agents (Claude Code and Codex) and the maintainer spend their effort on
content, and as little as possible on the mechanics around it. The target has five outcomes:

- **Start:** placement and identity are pre-decided.
- **Find:** content and prior answers can be found, with whether they still hold.
- **Write once:** each fact is written once.
- **Bookkeeping by tools:** tools keep the books while agents resume, adopt, close, move and
  retire work.
- **Reuse evidence:** optional shared tooling collects evidence and records its own conditions.

**Governing test.** A mechanism fails if it adds agent effort without clearly removing more.
Structure needs a programmatic action that reads through it, and a check needs a programmatic
fix.

**Variation axes:**

- new plans, companions and section shapes;
- concurrent plans;
- heading renames;
- closure and retirement;
- pins, toolchains or vendored sources drifting under evidence;
- policy wording changes;
- the second runtime: Codex has no path-triggered skills and usually no MCP here.

**Baseline.** Dirty `main` at `4c24721e6`, observed 2026-10-09. Plan 32's untracked
implementation is assessed as the current design: the strict reader, docs environment,
lifecycle overlay, explicit asset selection, producer artifact roles and observer.

**Evidence.** The [prepared bundle](../evidence/structured-documentation-lifecycle-2026-10-09/README.md)
contains the earlier inquiries and four later ones:

- earlier inquiries:
  - [content model and journeys](../evidence/structured-documentation-lifecycle-2026-10-09/content-model-and-journeys.md);
  - [mechanisms and consumers](../evidence/structured-documentation-lifecycle-2026-10-09/mechanisms-and-consumers.md);
  - [evidence reuse and collection](../evidence/structured-documentation-lifecycle-2026-10-09/evidence-reuse-and-collection.md);
- later inquiries:
  - **R1:** [plan format and section query](../evidence/structured-documentation-lifecycle-2026-10-09/plan-format-and-section-query.md);
  - **R2:** [agent runtime reach](../evidence/structured-documentation-lifecycle-2026-10-09/agent-runtime-reach.md);
  - **R3:** [evidence collection and staleness](../evidence/structured-documentation-lifecycle-2026-10-09/evidence-collection-and-staleness.md);
  - **R4:** [lifecycle tooling survey](../evidence/structured-documentation-lifecycle-2026-10-09/lifecycle-tooling-survey.md).

[Library alternatives](../evidence/structured-documentation-lifecycle-2026-10-09/library-alternatives.md)
and [representation exhibits](../evidence/structured-documentation-lifecycle-2026-10-09/representation-exhibits.md)
record the rejected format comparison. They are cited only for merge and rendering facts.

**Read directly by this reviewer:**

- Plan 32 and its [handoff](../evidence/workspace-content-lifecycle-2026-10-09/handoff.md);
- the [workspace](design_review_workspace-content-lifecycle_2026-10-09.md) and
  [target](design_review_workspace-content-lifecycle-target_2026-10-09.md) reviews;
- ADR-0168 and §24.4 / §24.4.1;
- `lifecycle.toml` and `site.toml`;
- `document_lifecycle.py` in full, and the relevant parts of `document_metadata.py`, `docs.py`,
  `adr.py`, `validation*.py`, `test_resources.py` and `library_catalog_db.py`;
- the plan, ADR, docs and turn-end recipes;
- the three indexes and the documentation guide;
- the `docs`, `decisions` and `tooling` rules;
- `create-plan` with `plan-structure.md`, and the uncommitted skill lifecycle paragraphs.

**My read-only scans** (*Measured*, this baseline):

- **Assets:** 0 of the roughly 55 files listed in `site.toml` asset declarations lack an inline
  link from a published page.
- **Current work:** the 14 `in-progress` plans (28, 28a–k, 32 and 33) are exactly the
  `current_work` selection.
- **Reviews:** 34 of 55 reviews have no inbound link from plans, indexes, skills, rules, dev
  guides or architecture sections. The prepared evidence counts 32 under a slightly different
  scope.

**Limits.**

- No repository file was written, and no repository test ran for this review.
- The R1–R4 probes ran on copies or in scratch. I inspected R1's outputs.
- Codex behaviour is *Interface-checked* only.
- One R4 trial (beads) sent default-on anonymous command metrics before they were disabled. No
  repository content was in their scope.

## 2. Where agent effort goes today

The system has a sound core:

- the strict byte-preserving reader (`document_metadata.py`) and the isolated `.venv-docs`;
- a publisher that derives navigation, scopes search and replaces the site atomically;
- ADR tooling and the register, the model of pre-decided mechanics;
- producer-owned disposal through validation receipts, sealed artifact roles, the resource
  ledger and the storage observer;
- plans and reviews that follow the skills' flexible guidance and stay readable without linting.

The effort goes elsewhere. The table separates real recurring costs from tolerable ones.

| Activity | What agents do today | Cost |
|---|---|---|
| Start a plan or evidence bundle | After `just plan`, add a self-owner exception, a selector per status table (exact heading and header strings) and a `current_work` glob. Make two `site.toml` edits per linked evidence file. Pick review and evidence names, dates, baseline and standard version by hand | Real: configuration restates the documents |
| Rename a heading or column | Edit the selector, or the whole `scope` query fails, reporting no item. Nothing routine runs it, so the break surfaces late | Real |
| Resume or update a plan | Read about 90 index lines for Plan 28 alone, then whole checkpoints (Plan 28's runs to about 275 lines). Find every place a packet change must propagate; EFF07 appears in 5 sections of Plan 33 (R1). Find what the skills say about the section being edited | Real: reading is content work, but finding *where* is mechanics |
| Know what is open across plans | `scope` is unused. Its regex leaves 59 of 228 items `unknown` and merges not-started with built-awaiting-qualification under "open". The output is 176 kB, and under `pse-env` admission it stalled 30 s and exited 125 (R2, *Measured*) | Real, and the tool does not remove it |
| Adopt a review or close a plan | Edit the [current-work](../../plans/README.md) and [review](../README.md) index prose, and append "Adoption handoff" text to reviews. The review index covers 8 of 55 reviews and still describes this review's rejected version | Real: restatements that drift |
| Retire a plan or review | Search for inbound links and repair them by hand (WC06 repaired 267). `retire-plan` always reports a literal blocker, scans inline links only and covers plans only | Real, at each closure |
| Ask "what do we already know about X" | Grep. SurrealDB material sits in about 9 bundles and was re-answered about 7 times in 5 days; later answers call earlier ones "leads only" (R3 §Q4) | Real: the costliest waste |
| Collect evidence | Hand-written harnesses (two 90-line `run.sh` files differ in 2 lines) and condition prose; 97 cited `build/` paths are mostly unregistered | Real |
| Change a shared policy sentence | The heuristics pointer is in 22 files and the ADR-0096 rule in 14. Versions are copied into 4 places. "Review cadence" is cited about 10 times and defined nowhere | Real, re-read on every load |
| Heading and front-matter variants; packet ID collisions; checkpoint heading variants; ADR index stale between turns; register rows | Nothing needs them uniform. All checkpoint headings already contain "checkpoint", and `turn-end` regenerates the ADR index | Tolerable: leave as is |

The mechanisms that add effort without clearly removing any are mostly recent, and nothing
routine consumes their output:

- the `lifecycle.toml` overlay: `doc_*` vocabulary and defaults, 48 exceptions, 48 table
  selectors, two regex status profiles and six relationships;
- the four `document_lifecycle` commands. No recipe, CI job or skill invokes them; every review
  and evidence file resolves to `protected-unknown`; the retirement preflight can never pass;
- `site.toml`'s `current_work` and asset lists, which reproduce exactly what plan status and
  links state;
- the instruction paragraphs, appended to three plan skills, the docs rule and the documentation
  guide, that tell agents to "maintain the plan's lifecycle metadata and native content
  bindings".

## 3. Change scenarios

| ID | Stimulus | Today | Target response |
|---|---|---|---|
| <a id="s01"></a>S01 | Resume an active plan and its companions | Read index prose and whole checkpoints, and reconcile companion statements | One command returns each active plan's checkpoint and non-finished rows as numbered source lines |
| <a id="s02"></a>S02 | A packet lands; the agent updates the plan | Find every section mentioning the packet; recall what the skills ask of a checkpoint or disposition | One command lists the sections and rows that mention the ID and quotes the skill paragraphs that guide them; the agent edits those lines |
| <a id="s03"></a>S03 | Add a plan, a companion or a new section shape; rename a heading | Exceptions, selectors and `current_work` edits; a rename breaks `scope` | No configuration edit. An unknown section kind gets no guidance rather than an error, and a rename breaks nothing (domain extension) |
| <a id="s04"></a>S04 | Adopt a review into a plan | Plan edits plus index prose and adoption notes in the review | Plan edits only. Adoption is visible because the plan links the review |
| <a id="s05"></a>S05 | Close a plan; retire it and its resolved review | Manual inbound-link search and repair; a preflight that cannot pass; index edits | `status: done` moves the plan to History automatically. Retirement repair is manual unless the optional command exists |
| <a id="s06"></a>S06 | "What do we already know about X?" before probing | Grep finds answers, but nobody can tell whether they hold | One command lists prior answers, each marked current, stale (with the cause) or uncaptured |
| <a id="s07"></a>S07 | A pin, toolchain or vendored source changes under evidence | Nothing notices. SurrealDB became a vendored path at the same version, POUNCE moved to a git fork, and the toolchain moved | Answers whose recorded used set changed report "stale because …". Rerunning one command refreshes them |
| <a id="s08"></a>S08 | Change a shared policy sentence | Edit up to 22 files; drift is already present | Edit one owner; the other places link to it |
| <a id="s09"></a>S09 | The same journeys from Codex, which has no path-triggered skills or command injection and usually no MCP here (mechanism substitution) | Skills name commands that sit behind admission and stall behind heavy work | One `just` command named in the shared skills, reaching both runtimes without admission |

## 4. Findings

| ID | Finding | Gates / foundations | Priority |
|---|---|---|---|
| [DL-F01](#dl-f01) | Configuration restates, and reinterprets, what documents already state | G1, G2, G7; AP-01, AP-02, AP-03, AP-04 | 1 |
| [DL-F02](#dl-f02) | Plans give no cheap route to "where must I update, and what should I consider" | AP-05, AP-06, AP-07 | 1 |
| [DL-F03](#dl-f03) | Evidence answers carry no comparable validity, so they are re-answered instead of reused | G6; AP-04, AP-07 | 1 |
| [DL-F04](#dl-f04) | Collection machinery is re-implemented per inquiry, and its outputs are invisible to their owner | AP-03, AP-07 | 2 |
| [DL-F05](#dl-f05) | Move and retirement bookkeeping is manual, and the preflight that exists cannot pass | G7; AP-03, AP-06 | 3 |
| [DL-F06](#dl-f06) | Shared policy is restated across instruction files, and some references resolve to nothing | G1; AP-04, AP-06 | 2 |

### <a id="dl-f01"></a>DL-F01: Configuration restates, and reinterprets, what documents already state

**Evidence.**

- **Plan currency is decided in about eight places:**
  - front matter `status`;
  - `current_work`;
  - collection scope;
  - index prose;
  - 21 self-owner exceptions;
  - table selectors;
  - a hard-coded filename regex in `aggregate()`;
  - checkpoint prose.
- **The delivered asset set is authored twice.** `site.toml` lists the files, and the pages link
  them. `validate_attachments` already rejects a link to an unlisted file, and every listed file
  is linked (*Measured*, §1).
- **Packet and finding state is copied and reclassified.** 48 selectors copy exact heading and
  header strings, and two regex profiles reclassify the author's prose. The
  [scope run](../evidence/structured-documentation-lifecycle-2026-10-09/mechanisms-and-consumers.md#2-scope-run-classification-quality-tested)
  shows the result (*Tested*):
  - "Implemented; …" is `unknown`, but the same text with "pending" is `open`;
  - "disproved as thrash; open as cost" is `open`;
  - done packets in done plans are `unknown`.
- **Instructions tell agents to keep this in step:** "update a binding when its native table
  changes".

**Consequence.**

- Ordinary edits reach owners unrelated to the content (AP-01).
- The query is coupled to presentation strings (AP-02).
- Status meaning the plan owns is independently reinterpreted (AP-03, AP-04, G2).
- Duplicated facts can disagree (G1).
- Unreliable classifications and a preflight that cannot pass are presented as authoritative
  (G7).
- None of it is routinely consumed, so the cost buys nothing.

**Correction.** Delete the overlay and the lists, and derive the two facts the publisher needs.

- **Plan scope follows `status`.**
  - `done` and `abandoned` go to History; anything else is Current, so an unknown value fails
    open and visibly.
  - Existing `site.toml` overrides stay for deliberate exceptions.
  - This is search scope, not authorization; done plans stay published.
- **Delivered assets are the non-Markdown targets linked from published pages.**
  - The containment, symlink and atomic-replacement rules stay.
  - Delivery stays independent of retention: an unlinked file is retained, not delivered.
  - To cite a large file without shipping it, link its permalink.
- **Delete:**
  - the `doc_*` fields and validators;
  - the defaults and exceptions;
  - the selectors, status profiles and relationships;
  - `document_lifecycle`'s commands;
  - the instruction paragraphs that route agents to them.
- **Keep:**
  - the strict reader and `.venv-docs`;
  - the closed legacy ADR scalar adapter, moved beside the ADR reader. It is the only
    `lifecycle.toml` content with a consumer.

**What changes for agents:**

- **Agents stop:** editing `lifecycle.toml` and `site.toml` when they start, rename or close
  work.
- **Agents newly do:** nothing.
- **Programmatic action:** two derivations in `scripts/docs.py`. The ADR status→scope mapping is
  extended to plans, and `validate_attachments` is inverted to produce the selection.

**Verification.**

- The published scope and asset set are unchanged at the switch.
- Adding a plan or linked evidence file, or renaming a heading, edits no configuration.
- No instruction mentions lifecycle metadata.

### <a id="dl-f02"></a>DL-F02: Plans give no cheap route to "where must I update, and what should I consider"

**Evidence.**

- Active plans total about 745 KB (R1, *Measured*).
- Resuming means reading index prose and whole checkpoints.
- Updating after a packet lands means finding every mention. EFF07 appears in five sections of
  Plan 33 (R1).
- What the skills ask of a checkpoint or disposition sits in paragraphs of long skills.
- The only aggregate tool classifies instead of locating (DL-F01). The index restatements are
  agents compensating.

**Consequence.**

- The cost of resuming and updating scales with document size, not with what changed (AP-06,
  AP-07).
- Where a change must propagate is a convention the agent remembers (AP-05).

**Correction.** A light structure, one query that reads through it, and guidance the skills
already own.

**1. Optional, invisible section kinds.**

- A level-2 heading may carry `<!-- kind: checkpoint -->`.
- The vocabulary has one owner: a Kind column in the `create-plan/references/plan-structure.md`
  Part table, plus `checkpoint` and `outcome`.
- Unknown kinds get no guidance, never an error.
- Existing plans need nothing. Title inference finds the checkpoint in all 21 plan files,
  packets in 9 of the 14 active plans, and dispositions wherever such a heading exists
  (R1, *Measured*).
- A 14-line converter adds markers on demand. It changes heading lines only and leaves every
  mdBook id identical (R1, *Tested*).
- `just plan` emits the markers. Its skeleton is generated from the Part table, which also ends
  today's competing skeleton.

**2. Guidance pairing without copies.**

- A skill paragraph that governs a kind carries `<!-- guides: checkpoint, dispositions -->`.
  The query quotes it with `file:line`.
- Nothing points into the skills from outside, so nothing dangles when they change.
- No reasoning guidance changes. Five markers covered the checkpoint case (R1, *Tested*).

**3. One query, `just plan-view` (name illustrative).** It parses on demand with the pinned
markdown-it-py and the existing table reader. Every line it prints is a numbered verbatim source
line, so the output is a valid edit target. Its views:

- `active`: each in-progress plan's checkpoint opening. 7.3 KB for all 14 plans.
- `<plan> --resume`: checkpoint text plus unfinished packet and disposition rows. 10–19% of each
  plan.
- `<plan> <kind> --guide`: a section with its breadcrumb and the paired guidance.
- `<plan> --mentions <ID>`: every section and row naming the ID.
- A roll-up line beside each disposition row: the first status word of the packets named in its
  owner cell, for example "F03: scheduled; owners EFF09 Integrated, EFF11 Planned".

**On R4's first-word classification versus R1's "tools gather, agents interpret".**

- The view hides a row only when its status cell *begins* with a terminal word: Done, Complete,
  Resolved, Superseded, Disproved, Retired or Abandoned.
- Everything else is shown, including prose the tool cannot read, so a miss can only add rows.
- Nothing is counted or reported as a state, and agents may pass their own filter.
- The regex profiles failed because they labelled and counted. A fail-open display filter
  does neither.

**4. Reach.**

- The command calls the `.venv-docs` interpreter outside workload admission (RC06).
- It defaults to one plan or the `active` digest, never the 176 kB full set.
- One sentence in `create-plan`, `execute-plan` and `plan-execution` names it. Both runtimes
  reach it through the shared skills, with no hook, SessionStart digest or MCP server (R2).
- A Claude-only `paths: docs/plans/**` skill is an optional extra.

**5. Indexes.**

- Delete the restated status prose from the two README indexes. Keep their routes and norms,
  and point to the command.
- The publisher may render the `active` list and a review list into the *staged* README at
  build time. The review list holds title, date, decision if present, and the plans linking each
  review. Nothing is committed, so nothing needs freshness checks.
- Stop appending adoption notes to reviews. The plan's link is the adoption record.

**What changes for agents:**

- **Agents stop:** reading whole plans to resume; hunting for an ID's mentions; re-reading skills
  for the applicable paragraph; editing index prose; writing adoption appendices.
- **Agents newly do:** run one command before updating a plan.
- **Programmatic action:** the parse-on-demand query (about 300 lines) and the marker-emitting
  scaffold.

**Verification.**

- `--resume` and `--mentions` work on every active plan, with and without markers, under a
  heavy native run.
- A heading rename breaks nothing.
- The skills' substantive text is unchanged apart from comments and the routing sentence.

### <a id="dl-f03"></a>DL-F03: Evidence answers carry no comparable validity, so they are re-answered instead of reused

**Evidence.**

- Finding answers works: rg finds all the SurrealDB material at once (R3 §Q4). Trusting them
  does not.
- Conditions are prose, and reuse is decided by prose labels ("leads only", "historical").
- [Observed drifts](../evidence/structured-documentation-lifecycle-2026-10-09/evidence-reuse-and-collection.md#3-lifecycle-state-stale-orphaned-consumer-retired)
  went unsignalled:
  - SurrealDB became a vendored path at the same version;
  - POUNCE moved from crates.io to a git fork;
  - the toolchain moved under two Arrow/DataFusion harnesses;
  - an efficiency probe's `input_scope_version` moved from 2 to 3.
- Two bundles (16 MB) survive only through publication and lifecycle configuration, with no
  inbound document link.

**Consequence.**

- Prepared answers are recomputed because their premises cannot be checked cheaply (AP-07; H12,
  H14).
- A reader can also reuse a stale answer with no signal (G6). The evidence model lacks the
  distinction reuse needs (AP-04).
- Refresh and delete decisions have no input but memory.

**Correction.** Make validity computed from captured conditions, never authored. This is the
Doorstop "suspect link" and DataLad "run record" pattern (R4).

- **`evidence run`** (DL-F04) writes a machine-generated `conditions.json` holding the probe's
  *used set*:
  - for Rust, dep-info source files and the `(name, version, source, checksum)` of each package
    in the probe's resolve;
  - for Python, imported repository files, files read and imported distributions (an audit
    hook);
  - the toolchain identity;
  - HEAD and dirty state, as context.
- **`evidence status`** prints `current`, `stale because <changed file / package source /
  version / toolchain>`, or `no capture`.
  - The three observed dependency drifts change a lock `source` or the `rustc -vV` commit hash,
    so the comparison catches them by construction (R3 §Q2).
  - A scan of used files for `*VERSION*` constants turns interpretation-version changes into
    semantic signals.
  - An unrelated `Cargo.lock` bump stays silent.
- **`evidence find <words>`** searches evidence and review Markdown with rg. Per hit it prints
  the title, date, linking documents and status. FTS5 or qmd are later options if paraphrased
  questions are missed.
- **`evidence new <slug>`** runs `find` first, then creates the folder. Prior answers surface at
  the moment of re-asking.
- **Refresh** is a rerun in place; Git keeps the prior result. An optional diff of the redacted
  decisive output reports "conclusion unchanged under new conditions", which directly replaces
  "leads only".
- **Deletion stays a decision.** `status` also shows bundles with no remaining inbound link.
  Nothing is gated, required, backfilled or deleted.

**What changes for agents:**

- **Agents stop:** re-answering to learn whether an answer holds; writing condition prose.
- **Agents newly do:** optionally run `find` or `new` before probing.
- **Programmatic action:** capture at run time, comparison on demand.

**Verification.** On fixtures:

- a path switch, a git source move and a toolchain change each produce their named stale line;
- an unrelated lock bump does not;
- `find` returns the SurrealDB bundles marked `no capture`.

### <a id="dl-f04"></a>DL-F04: Collection machinery is re-implemented per inquiry, and its outputs are invisible to their owner

**Evidence.**

- Two 90-line `run.sh` harnesses differ in 2 lines (*Measured*).
- Capture code is re-implemented in at least five bundles.
- Two packages kept outputs without their producer scripts.
- 97 distinct cited `build/` paths are mostly unregistered with the resource ledger.
- 16 MB of raw JSON and a 2.4 MB archive are committed into `docs/`.

**Consequence.**

- Each inquiry repeats harness work, and capture is inconsistent, which feeds DL-F03 (AP-03,
  AP-07).
- Producer-owned disposal cannot see that documents depend on these outputs.

**Correction.** One runner (`scripts/evidence.py`, about 300 lines; R3 option A) over the
toolchains' native script forms.

- **Rust probes** are single-file Cargo scripts, run on the pinned nightly with `-Zscript`.
  - A seeded copy of the repository lock passed through `resolver.lockfile-path` resolves
    wildcard requirements to exactly the repository pins. Probes never restate versions
    (R3, *Tested*).
  - Path dependencies on `pse-*` crates bring the vendored and git sources unchanged.
  - A shared evidence target directory bounds disk use.
  - Where production feature unification matters, the probe runs as an example or test target
    in an existing crate; the capture is the same.
- **Python probes:**
  - third-party-only probes carry a PEP 723 header and run with `uv run --script --locked`;
  - repository-internal probes use the repository venv.
- **Outputs.**
  - The bundle commits the probe, `conditions.json`, the redacted decisive output and the
    conclusion.
  - Raw outputs go to `build/evidence/<bundle>/<run>/`, registered with the existing ledger so
    disposal stays producer-owned.
- **Cleanup.** The two `run.sh` files are deleted once their probes run through the runner.
  Existing committed outputs stay where they are.

**What changes for agents:**

- **Agents stop:** writing harnesses and capture code.
- **Agents newly do:** optionally wrap a probe in one command.
- **Programmatic action:** the runner, composed from Cargo and uv script support, dep-info,
  `cargo metadata`, an audit hook and ledger registration.

### <a id="dl-f05"></a>DL-F05: Move and retirement bookkeeping is manual, and the preflight that exists cannot pass

**Evidence.**

- `retire-plan` appends the literal blocker "owner release not established" to every
  candidate, scans inline links only, and covers plans only.
- WC06 repaired 267 links by hand.
- `adr.py` encodes the ADR review-relocation rules, but only as lint.

**Consequence.**

- Each closure repeats a mechanical procedure (AP-03, AP-06).
- The preflight advertises a capability it lacks (G7).

**Correction.**

- Delete `retire-plan` with the overlay (DL-F01). Retirement is then no worse than today.
- **Optional and lowest priority:** a `docs-retire` / `docs-move` command, OpenSpec's "archive as
  one command" pattern (R4). It would:
  - compute inbound references (links including fragments, front-matter path lists and ADR
    `review:` fields);
  - rewrite them byte-preservingly to the new path, the surviving owner or the HEAD permalink;
  - report constructed paths in scripts without editing them;
  - list bundles that would lose their last link;
  - then move or remove the named files on that explicit invocation.
- The IWE trial shows both that the operation is mechanical and what to avoid: it dropped
  fragments and re-serialized the file (R4, *Tested*).

**What changes for agents:**

- **Agents stop:** hand-repairing links.
- **Agents newly do:** run one command instead of a search.
- **Programmatic action:** a small rewriter on the existing token walk and the `adr.py`
  relocation helpers.

### <a id="dl-f06"></a>DL-F06: Shared policy is restated across instruction files, and some references resolve to nothing

**Evidence.**

- The heuristics pointer paragraph is in 22 files (*Measured*), and the ADR-0096 rule in 14.
- The checkpoint definition has at least six phrasings.
- Standard versions are copied into four places the binding says it does not copy to.
- About ten files cite a "review cadence" nobody defines.
- The library-research role cites evidence locations that "AGENTS.md names"; it names none.

**Consequence.**

- Each load re-reads the same paragraphs.
- Each policy change is an N-file edit, and drift is visible (G1, AP-04).
- Undefined references send agents searching (AP-06).

**Correction.** Consolidation by deletion.

- Keep each policy at its owner: the standard and binding for the heuristics pointer; ADR-0096
  and §24.4.1 for retention.
- Each process skill keeps its own operative pointer. Its reasoning guidance is out of scope and
  unchanged.
- Echoes in READMEs, rules, roles, §24.4 and the binding become links.
- `standard.toml` owns versions and gains the heuristics version.
- Define the cadence in one binding sentence, or delete the references.
- Name the evidence root once.
- Instruction generators such as ruler and rulesync do not deduplicate prose (R4), so no tool
  helps here.

**What changes for agents:**

- **Agents stop:** re-reading echoes; making N-file edits.
- **Agents newly do:** nothing.
- **Programmatic action:** none needed.

## 5. Recommended target

### 5.1 How the pieces fit

The recommendations share one stance: tools *gather and keep books*, and agents *interpret*.
Each adds structure only where a named tool reads through it.

| Structure | Read by | Authored by |
|---|---|---|
| Plan front matter `status` (existing) | Publisher scope; `plan-view active` | Agent, as today |
| Inline links (existing) | Asset delivery; "linked from" lists; evidence consumers; optional retire | Agent, as today |
| `<!-- kind: … -->` on plan headings (new, optional) | `plan-view` | `just plan` for new plans; legacy plans use title inference |
| Kind column in the `plan-structure.md` Part table (new) | `plan-view`, `just plan` skeleton | Once, by the skill owner |
| `<!-- guides: … -->` above skill paragraphs (new, a handful) | `plan-view --guide` | Once, by the skill owner |
| `conditions.json` in evidence bundles (new) | `evidence status` / `find` | The runner, never an agent |

**What is not proposed:**

- markers inside reviews;
- closed status vocabularies;
- a schema or validator;
- generated committed indexes;
- a new hook or SessionStart digest;
- an MCP server;
- an always-on check.

The only checks that remain are the existing ones: rendering, offline links, ADR lint and the
register. Each already has a direct fix.

### 5.2 Plans

Plans keep their prose, tables and flexible organization. R1 tested data encodings for long
reasoning, and each fails silently or breaks editing (*Tested*):

- YAML turns an unindented paragraph into a new key.
- TOML binds an appended key to the last table.
- JSON breaks exact-string edits.

A heading per element with a `- State:` line merges cleanly where adjacent table rows conflict.
It is available, not prescribed. Concurrent plans keep one status owner each: the query reads
each plan's own tables, and the roll-up shows an owner's status without copying it.

### 5.3 Reviews

Leave reviews as they are. A review is written once and read at adoption, where the
plan-creation process reads it whole anyway. Its findings and rule impacts already have
anchors that plans link.

The two recurring costs are both addressed without touching review format:

- **"Which reviews exist and which are adopted"** is answered by the publish-time list (linking
  plans computed from links) and by `evidence find`, which also searches reviews.
- **Adoption notes appended to reviews** stop.

A review scaffold would save little, because the binding's filename pattern is the whole decision.

### 5.4 Evidence and its preservation obligation

Review- and plan-produced bundles keep one authored root, `docs/design_review/evidence/`.
Capability-map evidence stays with `evidence-regen`.

Staleness is advisory. It never deletes, releases, gates or relabels anything. A stale answer
remains retained evidence of what held under its recorded conditions, and neither completion
nor a computed signal authorizes deleting evidence, caches or state. Raw outputs registered in
the ledger inherit its producer-owned protections, and unregistered legacy outputs stay
protected as today.

## 6. Outward tooling and library fit

R4 surveyed the agent lifecycle field and ran hands-on trials of Beads, Backlog.md, OpenSpec,
IWE and mdq/mq on copies of Plan 33. It covered:

- trackers;
- spec frameworks: Spec Kit, OpenSpec, Kiro, BMAD;
- Markdown knowledge and link tools: Basic Memory, Marksman, IWE, qmd;
- query tools;
- docs-freshness tools: Swimm, menard, cogapp;
- ADR and traceability tools: Doorstop, sphinx-needs;
- evidence tools: DataLad, DVC, Quarto, MLflow, Sacred;
- instruction generators.

Little is adopted. The tools that remove effort compute views from what is already written,
but most of the field adds compliance instead:

- **A second status store.** Beads uses a Dolt database, emits an 842-word rules primer and
  auto-committed on init.
- **CLI-only edits.** Backlog.md requires them, and its sequential IDs collided across branches
  (*Tested*).
- **Prompt protocols.** Spec Kit, Conductor and BMAD have agents follow them.
- **Rewritten authored files.** Basic Memory rewrites front matter, IWE drops fragments and
  re-pads tables, and mdq and mq re-render tables.

| Capability | Candidate | Fit and limits | Recommendation |
|---|---|---|---|
| Plan section query | markdown-it-py 4.2.0 (already pinned in `.venv-docs`) | Token line maps give verbatim, numbered edit targets; shares the existing reader | Use (DL-F02) |
| | mdq 0.10.0 / mq 0.9.2 | Good read-only selectors; no kinds, no line numbers, tables re-rendered; one formatting error on a large section | Optional viewers; borrow the selector idea |
| Rust probes | Cargo frontmatter scripts (`-Zscript` on the pinned nightly) + `resolver.lockfile-path` | Exact repository pins with no restatement; cargo-script is still unstable; per-script feature unification | Use (DL-F04); revisit at stabilization |
| Python probes | PEP 723 + `uv run --script --locked` | Checked dependency declaration and drift detection; cannot cheaply provide the maturin package | Use for third-party-only probes |
| Validity | Doorstop suspect links, DataLad run records | Right shape; tools too format-bound or heavy (git-annex) | Pattern only (DL-F03) |
| | DVC 3.67.1 | Staleness only for declared deps; `.dvc` cache and state | No |
| Output drift on refresh | insta 1.49.0 (already pinned) | Shows *what* changed after a rerun | Optional for decisive outputs |
| Prior-answer search | rg / SQLite FTS5 / qmd 2.8.3 | rg suffices; qmd adds paraphrase recall with about 2 GB of models | rg now; qmd if paraphrases are missed |
| "What cites this" | IWE, Marksman | Query is good; writes are unsafe (IWE) | Optional LSP or query only |
| Link repair | none byte-preserving found | IWE proves the operation is mechanical but reformats | Small bespoke rewriter if DL-F05's option is taken |
| Regenerated blocks | cogapp, markdown-magic | Programmatic fix exists | Not needed: lists render at publish |

**G8 passes.** The bespoke code that remains is bounded: a query of about 300 lines and a
runner of about 300 lines. Both sit on pinned libraries and native toolchain features. No
library clearly provides either contract.

## 7. Alternatives

| Alternative | Effort removed | Effort added | Assessment |
|---|---|---|---|
| Current baseline: overlay plus `scope` | Little; the output is unconsumed | Selectors, exceptions, lists, regex upkeep, instruction duties | Fails the governing test |
| Rejected first version: authored markers, closed vocabularies, schema, validating reader | Much, in principle | Marker and vocabulary compliance on every element; diagnostics without fixes | Rejected by the maintainer; superseded here |
| Data-encoded plans (YAML, TOML) with projections | Exact field edits | Indentation and table-ordering hazards that fail silently; a projection for publication (R1) | No |
| External tracker (Beads, Backlog.md) | `ready` and roll-up views | A second store, CLI-only edits and rule primers (R4) | No; take the computed views |
| **Simplest viable: deletion only (DL-F01, DL-F06)** | All configuration restatement | Nothing | Necessary but insufficient: resume, update and evidence validity stay unaddressed (S01, S02, S06, S07) |
| **Recommended:** deletion plus `plan-view` plus the evidence runner (link repair optional) | Configuration, index upkeep, whole-plan reading, harness writing, re-answering | One command before plan updates; optional runner use | Removes clearly more than it adds |

The recommendation is reopened if either of these happens:

- agents in practice do not run `plan-view`, which can be observed in a few sessions; the
  optional Claude path skill then becomes the next lever, then an inject-only edit hook (R2);
- the used-set staleness proves too noisy or too blind on real probes.

## 8. Architectural assessment and gates

The verdicts apply to the content-management responsibilities in scope.

| Foundation | Scenario and evidence | Verdict |
|---|---|---|
| AP-01 Separation of concerns | S03: adding a plan or renaming a heading edits `lifecycle.toml` and `site.toml`, owners unrelated to the content (DL-F01) | Violated |
| AP-02 Stable contracts | The scope query depends on exact heading and header strings (presentation leaks into the contract). The reader, publisher and ADR contracts are sound | Violated (scope overlay only) |
| AP-03 Composition | The regex profiles reinterpret status meaning the plan owns; retirement is an end-to-end manual procedure repeated per closure; harnesses are copied per inquiry (DL-F01, DL-F04, DL-F05) | Violated |
| AP-04 Domain model and authority | Model adequacy: the authored plan model (status, headings, tables, links) is adequate for the needed operations, but the evidence model lacks validity conditions as data (DL-F03). Authority: currency and the delivered set have two writable owners, and policy has up to 22 (DL-F01, DL-F06) | Violated |
| AP-05 Explicit structure | Where state lives and where changes must propagate are conventions the agent remembers; evidence conditions are prose (DL-F02, DL-F03) | Violated |
| AP-06 Local reasoning | Resuming requires reading whole plans (about 745 KB active); one bad selector fails the whole query; references to undefined cadence (DL-F02, DL-F06) | Violated |
| AP-07 Execution fit | The workload is agent operations on content. Answers are recomputed because validity cannot be checked (H12, H14); restated guidance is re-read on every load; the 176 kB scope output and the 30 s admission stall misfit a read-only context query (R2, *Measured*) | Violated |

| Gate | Result | Evidence |
|---|---|---|
| G1 Authority | Fail | Plan currency and the delivered asset set each have two writable definitions; the indexes restate plan and review state, and one now contradicts this review (DL-F01, DL-F06) |
| G2 Semantic fidelity | Fail | Regex profiles reinterpret status prose; identical meanings land in different states (DL-F01) |
| G3 Validity | Pass | The strict reader rejects malformed metadata; the publisher rejects escaping or symlinked assets |
| G4 Hidden behavior | Pass | Lifecycle tools are read-only; publishing replaces the site atomically |
| G5 Consistency and recovery | Pass | Atomic site replacement; ledger reservations, borrowers and sealed roles |
| G6 Transformation and reuse | Fail | Evidence is reused or rejected by prose; recorded conditions drifted unsignalled (DL-F03) |
| G7 Truthful capability claims | Fail | `retire-plan` presents a preflight that cannot pass; `scope` counts look authoritative but misclassify (DL-F01, DL-F05) |
| G8 Library leverage | Pass | Pinned parsers and native script support are used; the survey found no library owning the remaining contracts |
| G9 Architectural fitness | Fail | Follows AP-01 to AP-07 |
| Profile gates | Not applicable | No scientific model or execution contract in scope |

## 9. Verification and evidence limits

| Claim | Label | Basis | Gap |
|---|---|---|---|
| Configuration restates plan status and links | *Measured* | My scans: 14 in-progress plans equal `current_work`; 0 listed assets unlinked | — |
| Regex scope misclassifies and is unconsumed | *Tested* / *Interface-checked* | Prepared scope run (228 items); grep of recipes, CI and skills | — |
| `plan-view` locates update targets with paired guidance | *Tested* on copies (R1) | `planq.py` probe: `active` 7.3 KB, resume 10–19%, `--mentions`, `--guide`, 14-line converter, mdBook ids unchanged | Production tool not built; live plans may have changed since the copies |
| Both runtimes reach a `just` command named in skills | *Interface-checked* (R2) | Claude and Codex docs; Codex binary strings | Whether Codex agents run it unprompted is unobserved |
| Admission stalls a read-only query | *Measured* (R2) | 30.07 s, exit 125, under an exclusive native run | — |
| Used-set capture catches observed drifts | *Tested* in part (R3) | Lock `source` and `rustc -vV` capture exercised; Python capture of one probe | Build-script inputs and server-version probes (one author line) not exercised |
| Cargo scripts reproduce repository pins | *Tested* (R3) | Seeded lockfile on the pinned nightly | `-Zscript` instability |
| Outward tools add compliance | *Tested* / *Interface-checked* (R4) | Five trials; documentation for the rest | qmd recall not trialled |
| The remedies remove more effort than they add | *Proposed* | Reasoning from S01–S09 | Settled only by use |

**Landed rather than relocated.** Each finding names its own verification. Across all of them,
a correction has landed only if no new lint, hook, gate or stored index appears and no
instruction asks agents to maintain lifecycle metadata.

**Material uncertainties:**

1. **Uptake.** The query helps only if agents run it. Codex skill routing is probabilistic (R2).
   This is settled by observing ordinary plan updates; the fallbacks are named in §7.
2. **Checkpoint length.** Very long checkpoints (Plan 28e's own is 167 lines) are a writing
   habit no tool removes; `--own` trims subsections only.
3. **Staleness granularity.** It is untested for probes with native build scripts or remote
   servers.
4. **Done plans and History.** Moving done plans to History by status is search scoping. If the
   maintainer wants a done plan with a live handoff in Current, an override expresses it.
5. **The terminal-word set.** It is a display convenience. Its fail-open design bounds the cost
   of a miss to extra rows.

## 10. Rule impacts and disposition

These would change only after operator confirmation in plan creation.

| ID | Rule and location | Proposed change | Depends | If kept |
|---|---|---|---|---|
| <a id="rc01"></a>RC01 | Blueprint §24.4.1 (defaults, exceptions, content bindings); [ADR-0168](../../adr/0168-workspace-content-lifecycle.md) (proposed); Plan 32 *Target design*; [documentation guide](../../dev/documentation.md) *Metadata and aggregate scope* | Remove the `doc_*` vocabulary, defaults, exceptions, scope bindings, status profiles, relationships and `document_lifecycle` commands. Keep the strict reader, docs environment and relocated legacy ADR adapter. Revise ADR-0168 in place and amend §24.4.1 through the design route | DL-F01, DL-F05 | The overlay stays a parallel duty. At least remove the instruction paragraphs; DL-F01 stays open |
| <a id="rc02"></a>RC02 | Documentation guide ("select their exact paths in its `assets` declaration"; current work selected "because a plan's lifecycle field is not a resume instruction"); `.claude/rules/docs.md`; Plan 32 *Discovery and publication* | Derive plan Current/History from status (overrides kept) and delivered assets from links | DL-F01 | Plan starts, closures and evidence files keep needing `site.toml` edits |
| <a id="rc03"></a>RC03 | Lifecycle paragraphs in `create-plan`, `execute-plan`, `plan-execution` and `.claude/rules/docs.md`; the AGENTS.md documentation route | Replace them with one sentence naming `plan-view`, and `evidence find` in the review-planning skills. Reasoning guidance unchanged | DL-F01–F03 | Agents are told to maintain removed machinery, or never find the commands |
| <a id="rc04"></a>RC04 | `plan-structure.md` Part table; `just plan` skeleton | Add a Kind column and a few `<!-- guides: … -->` comments; generate the skeleton from the table with markers | DL-F02 | The query falls back to title inference with no paired guidance |
| <a id="rc05"></a>RC05 | [Current-work](../../plans/README.md) and [review](../README.md) index conventions | Delete restated status and adoption prose; render lists at publish; no adoption notes in reviews | DL-F02 | Hand edits and drift continue |
| <a id="rc06"></a>RC06 | AGENTS.md *Start here* (workload recipes run through `pse-env`; only `just activity` runs directly); `.claude/rules/tooling.md` *Fixed observation* | Extend the observer exception to the read-only plan query and `evidence status`/`find` | DL-F02, DL-F03 | The query stalls behind heavy work (R2) unless run as a bare interpreter call |
| <a id="rc07"></a>RC07 | Binding *Reviews in this repository* (evidence location); §24.4 and the library-research role ("locations AGENTS.md names") | Name `docs/design_review/evidence/` as the authored root; runner raw outputs go under `build/evidence/` | DL-F03, DL-F04, DL-F06 | The location stays implicit |
| <a id="rc08"></a>RC08 | Restated policy in AGENTS.md, rules, roles, the docs and review READMEs, §24.4, the binding and the documentation guide; copied versions; "review cadence" references | One owner per policy, with links elsewhere. `standard.toml` owns versions, including heuristics. Define the cadence once or remove it. Skills keep their own pointers | DL-F06 | N-file policy edits and drift persist |

**Disposition owner.** Plan 32 retains WCL-F01–F05. Whether this review's remedies resolve or
supersede parts of their corrections is Plan 32's disposition. **Proposed:** close Plan 32 on its
representation-independent work and start a new plan. The new plan owns:

- DL-F01–F06;
- confirmation of RC01–RC08;
- the deletion of the overlay as its first packet.

Plan 32 keeps the strict reader, the docs environment, publisher atomicity, producer artifact
roles, the observer, the pre-28 removal and its WC08 qualification of those. Its WC02 and WC03
are re-scoped to exclude the overlay and the explicit lists.

Revising Plan 32 instead is viable, and gives the overlay's author ownership of its removal. A
coordinator-plus-companion arrangement adds coordination without a need for it.

## 11. Decision

- **Behavioral and semantic adequacy: not adequate.** G1, G2, G6 and G7 fail. G3, G4, G5 and G8
  pass.
- **Architectural fitness: not fit.** G9 fails on all seven foundations; AP-02 only for the scope
  overlay.
- **Overall: Revise.**

The recommended target is *Proposed*. Its feasibility rests on probes run on copies and in
scratch, not on an implementation.

| Priority | Change | Findings / scenarios | Acceptance evidence | Owner |
|---|---|---|---|---|
| 1 | Delete the overlay, lists, commands and instruction paragraphs; derive scope and assets in the publisher | DL-F01; S03, S04, S05 | Unchanged published scope and asset set at the switch; no configuration edit when adding a plan, adding evidence or renaming a heading | New plan |
| 1 | `plan-view` with kind inference and markers, guidance pairing, fail-open open-rows view and roll-up; skill sentence; observer exception | DL-F02; S01, S02, S09 | Resume and mentions on every active plan under heavy load, from both runtimes; skills' substantive text unchanged | New plan |
| 1 | Evidence runner: run, status, find, new | DL-F03, DL-F04; S06, S07 | Fixture drifts reported with named causes; unrelated lock bumps silent; `run.sh` files deleted | New plan |
| 2 | Consolidate restated guidance | DL-F06; S08 | One owner per policy sentence; cadence defined or removed | New plan |
| 3 | Optional link-repairing retire and move | DL-F05; S05 | A replay of a WC06-style retirement rewrites links, fragments included, byte-preserving | New plan, if selected |

**Prerequisite order.**

- The deletion has no prerequisite. Because nothing routine consumes `scope`, it can land before
  `plan-view`.
- The legacy ADR adapter must move before `lifecycle.toml` is removed.
- RC06 must be confirmed before the query is wired into skills.
- The evidence runner and guidance consolidation are independent of the plan work.

**Next consequential decision.** The operator's confirmation of RC01 and RC02, which retire
Plan 32's overlay and the publication lists. Every other recommendation is additive and optional
to adopt.
