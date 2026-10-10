# R1: Light structured plans, section queries and paired guidance

**Role, sources and baseline.** This is library and tool research for the structured
documentation review. It recommends nothing. The coordinator decides adoption.

- **Scope.** The maintainer redirected the work mid-assignment to four questions:
  - (a) a light structured plan format;
  - (b) querying sections and elements by kind and state;
  - (c) pairing each section kind with process-skill guidance;
  - (d) migration "going forward".

  The original Q1–Q4 (link repair, regenerate-in-place, scaffolds, backlinks) were dropped. Q5
  and Q6 survive only where they serve (b).
- **Baseline.**
  - Checkout: `main` at `4c24721e6`, dirty, with untracked Plans 32/33 and the structured-documentation
    evidence present.
  - All plans and process skills were **copied** into scratch and queried there. No repository
    file was written.
- **Builds on.** The existing untracked evidence folder
  `docs/design_review/evidence/structured-documentation-lifecycle-2026-10-09/`:
  `library-alternatives.md` and `probes/README.md`, with probes P1 and P2 and the merge exhibit.
  Facts established there are cited, not re-run.
- **Sources.**
  - Context7: mdBook (`/rust-lang/mdbook`), Pagefind, Marksman, yaml-language-server and taplo.
  - Release metadata: GitHub releases for mdq, yq and mq; npm.
  - Upstream `--help` and module help for mdq and mq.
  - Local runs.
- **Conditions for every run.**
  - Host: Linux 7.0.0-38-generic x86_64, on 2026-10-09.
  - Python: CPython 3.14.7 in a uv venv with `markdown-it-py==4.2.0`, `mdit-py-plugins==0.6.1`,
    `mdurl==0.1.2`, `tomlkit==0.15.1` and `ruamel.yaml==0.19.1`, run with `python -I`.
  - Tools: mdBook v0.5.4, git 2.43.0 and jq 1.7.
  - Release binaries: mdq 0.10.0 (2026-03-22), mq 0.9.2 (2026-09-28) and yq v4.54.1 (2026-09-29).
- **Labels.**
  - *Tested*: a script compared or asserted an outcome under those conditions.
  - *Measured*: an output was observed and recorded.
  - *Interface-checked*: from documentation or source only.

---

## (a) Plan formats that keep long reasoning prose easy to read and edit

### What the probes show

**1. Section kind markers in Markdown (mdBook 0.5.4, *Tested*, fixture `a scratch mdBook fixture (not retained)`).**

| Marker form | Rendered `<h2>` | Raw-read noise | Notes |
|---|---|---|---|
| `## Current checkpoint {#checkpoint role=checkpoint}` | `<h2 role="checkpoint" id="checkpoint">`: pulldown-cmark emits **every** `key=value` as an HTML attribute | braces visible | `role` is an ARIA attribute, so use another key. `{#id kind=x}` or `{#id data-kind=x}` parse cleanly with `mdit_py_plugins.attrs.parse` (*Tested*). On GitHub, braces stay visible and the anchor differs (prior *Measured*, `library-alternatives.md` table). |
| `## Finding dispositions <!-- kind: dispositions -->` (inline comment) | id `finding-dispositions`, **unchanged**; comment kept inside the heading HTML, invisible | one short comment | No escaped text leaked into `fixture.html` or `print.html` (*Tested*, grep). |
| `<!-- kind: packets -->` on the line above the heading | comment passes through; id unchanged | one line | Detaches if someone inserts text between the comment and the heading. |
| `{#verification .role-verification}` | `class="role-verification"` | braces | Class form works too. |
| `{#p02}` inside a table cell | literal text | — | No attributes in cells; the existing `<a id="eff00"></a>` row anchors work. |

**2. One plan slice in four encodings (`probes/plan-query/formats_gen.py`, output
`results/formats.txt`).** Each encoding holds two packets with multi-paragraph bodies, a state
field and a checkpoint. The scenarios are edits that two agents could make concurrently. The
merge column is the number of conflicts from `git merge-file`.

| Property | Light Markdown (heading per element + `- State:` line) | YAML, `\|` block scalars | TOML, `'''` literal strings | JSON |
|---|---|---|---|---|
| A two-line prose span copied as read is found by exact string (*Tested*) | yes | **no**: indentation is part of each line | yes: the body bytes are identical to Markdown | **no**: `\n` and `\"` escapes |
| A quoted phrase is found (*Tested*) | yes | yes | yes | **no** |
| S-a: two edits to different paragraphs of one element (*Tested*) | 0 conflicts | 0 | 0 | **1**: the body is one physical line |
| S-b: state edits on adjacent elements (*Tested*) | 0 | 0 | 0 | 0 |
| S-c: state edit and body edit on the same element (*Tested*) | 0 | 0 | 0 | **1** |
| Programmatic one-field state edit (*Tested*) | line splice at the parser's line map | ruamel round-trip: 1 line changed | tomlkit: 1 line changed | `json.dumps` rewrites the file |
| Authoring hazards (*Tested*) | none new | Unquoted `state: in progress: x` gives `ScannerError`. An unindented appended paragraph gives `ScannerError`. An unindented `Note: …` line silently becomes a **new top-level key**. | A top-level key appended after `[[packets]]` silently binds to the **last packet** (`checkpoint` landed in `packets[-1]`). | Every newline and quote must be escaped. |
| mdBook 0.5.4 | native | needs a projection in `scripts/docs.py` staging | needs a projection | needs a projection |
| Raw reading by agents | prose as written | prose indented under keys | prose verbatim between `'''` fences | prose as escaped one-liners |

Conflict behaviour in tables is already known. In the prior merge exhibit, adjacent-row status
edits and two-field edits of one row **conflict** in table encodings (S3/S4 in
`probes/README.md`). Heading-per-element, typed blocks and YAML do not conflict there.

**3. What a schema gives for free (*Interface-checked*).**

- **Association:**
  - YAML: the `# yaml-language-server: $schema=<path>` modeline;
  - TOML: taplo's `#:schema ./x.json` directive;
  - JSON: a `$schema` key.
- **What it buys:** editor completion, and hover of `description`/`markdownDescription`. Diagnostics
  come from `taplo check --schema …` or the language server.
- **For agents:** nothing, unless a language server or validator is run. Running one produces
  diagnostics without fixes, which is the lint shape the maintainer rejected.
- **The one useful free asset** is the schema's `description` text. But that would be a second
  copy of the skills' guidance (see (c)).
- **Markdown equivalent:** front matter is already read by `scripts/document_metadata.py`; no
  schema layer is needed.

### Interpretation

- **Long reasoning prose stays best in Markdown.**
  - It is the only encoding where the agent's exact-string edits, raw reading, mdBook and Git
    diffs all work unchanged.
  - TOML `'''` is the only data encoding that keeps prose bytes verbatim. Its silent
    table-binding hazard and the need for projection make it a fit for small records, not for
    plans.
  - YAML's hazards are silent in the worst case.
  - JSON fails exact-string editing outright.
- **Which marker.** The inline comment `<!-- kind: X -->` is the lightest marker measured:
  invisible in mdBook, id-neutral, one token for a raw reader. Use an explicit `{#id}` only where
  a stable anchor across heading renames matters, with `kind=` rather than `role=` if attributes
  are used.
- **Elements.**
  - Table rows with `<a id>` anchors stay the compact overview.
  - A heading per element, with a leading `- State:` line, gives merge-friendly multi-line
    elements that mdBook renders natively.
  - That avoids the colon-fence containers, which garble under mdBook's default definition lists
    (P1/P2).

---

## (b) Query tooling: fetch sections and elements by kind and state

### Off-the-shelf Markdown query CLIs (*Tested* on a copy of Plan 33)

| Tool | Section by heading | Table rows | Byte fidelity of output | Line numbers | Neighbours and relations |
|---|---|---|---|---|---|
| **mdq 0.10.0** (Rust, single binary) | `mdq '# Current checkpoint'`, which includes subsections, 15 ms | `:-: /Packet\|Progress/ :-: /EFF1/`. The row matcher matches **any cell**: EFF00 was selected because its text mentions EFF10. No per-column filter. | prose identical except the trailing blank line; tables **re-padded**: 29 of 33 lines changed | no | no |
| **mq 0.9.2** (jq-like, Rust) | `mq -A 'import "section" \| section::find(self, "Current checkpoint", 2) \| section::all_nodes()'` | via node filters | prose identical; tables re-rendered: 30 of 33 lines changed | not exposed | **yes**: `section::tree::breadcrumb`, `prev_sibling`, `next_sibling`, `ancestors` (*Interface-checked*: `mq help section`) |
| **yq v4.54.1** over YAML (and `-p toml`) | n/a | `.packets[] \| select(.state != "done*") \| {id, line: (. \| line), state}` | values re-serialized | **yes** (`line` operator, *Tested*: 4 and 14) | via paths |
| **jq 1.7** over JSON | n/a | `select(.state\|startswith("done")\|not)` | values | no | via paths |

**Interpretation.**

- **mdq and mq** are good read-only viewers of prose sections. They cannot produce exact-string
  edit targets for table rows, and neither knows a section's kind.
- **yq and jq** are exact over data, but presuppose the data encodings rejected in (a).

### A small parse-on-demand CLI (probe `probes/plan-query/planq.py`, 318 lines)

The probe uses markdown-it-py `commonmark` with tables and `front_matter_plugin`. It reads
heading and row line maps from the token stream; `tr_open.map` gives each body row its source
line (*Tested*). Every output line is a **verbatim source line with its number**, so the result
is a valid `Edit` target.

**How a section's kind is resolved:**

1. a heading attribute `kind=`/`data-kind=`;
2. otherwise an inline `<!-- kind: X -->`;
3. otherwise a comment on the line above;
4. otherwise, for level-2 headings only, a legacy title alias.

**Commands:**

- `outline FILE`: headings with kind, how the kind was found, line range and row count.
- `section FILE KIND|ANCHOR [--own] [--guide]`: a context-rich slice, consisting of:
  - the heading path (breadcrumb);
  - previous and next sibling headings with line ranges;
  - IDs mentioned in the section (EFF07, F02, ADR-0169 …);
  - outbound links;
  - with `--own`, the text up to the first subsection, plus the subsection list (so dated
    handoff history is listed, not dumped);
  - with `--guide`, the paired guidance from (c).
- `rows FILE KIND --where 'COLUMN~regex' / 'COLUMN!~regex'`: rows filtered by a named column.
  The **agent** supplies the predicate; the tool classifies nothing. Tables without the column
  are skipped.
- `elements FILE KIND --where 'state!~^done'`: heading-per-element children and their leading
  `- Key: value` lines.
- `mentions FILE ID`: every section and row line that mentions an ID, marked as prose or row.
  This answers "sections affected by packet X".
- `active DIR`: for each plan whose front matter says `status: in-progress`, the first
  paragraph of its checkpoint section.

**Measured results (copies of the live plans).**

- **Plan 33 open packets.** `rows … packets --where 'progress!~^(Integrated|Implemented)'`
  returned EFF00, EFF10 and EFF11 as source lines 256, 266 and 267.
- **EFF07 mentions.** `mentions … EFF07` found 5 sections: migration row 92; packets prose and
  rows 263, 264, 270 and 276; a boundary row 318; disposition row 397; checkpoint prose 424
  (`results/mentions-33-EFF07.txt`).
- **Active plans.** `active` over 22 plan files found all **14** in-progress plans and a
  checkpoint in each. It returned 7,265 bytes, against 744,539 bytes of active plan text, with
  the first paragraph capped at 400 characters (`results/active.txt`).
- **"Resume" bundle.** This is the own checkpoint text, plus packet rows whose progress does not
  start with Integrated/Implemented/Done/Complete, plus disposition rows not starting with
  Resolved.

  | Plan | Bundle | Whole file | Share |
  |---|---|---|---|
  | Plan 33 | 9,983 B | 51,240 B | 19% |
  | 28e | 15,490 B | 158,787 B | 10% |
  | Plan 28 | 17,369 B | 105,705 B | 16% |

  28e's own checkpoint is still 167 lines. That is a writing habit, with current state mixed
  with older notes, and no tool removes it.
- **Cost.** 65 ms per invocation on 28e, mostly interpreter start-up; 1.16 s for 22 sequential
  `outline` processes.

**Precision of legacy title inference (*Measured*, `results/outline-all-plans.txt`).**

- **Correct everywhere it matters for resuming:**
  - checkpoint in all 14 active plans and in all 21 non-README plan files (titled "Current
    checkpoint", "Checkpoint and next step", "Checkpoint" or "Open items and current
    checkpoint");
  - packets in 9 of the 14 active plans (titled "Work packages", "…and dependencies" or
    "Execution sequence and readiness"). 28h–28k and 32 have no packets-like level-2 heading;
  - dispositions in 3 of the 14 active plans (28, 32 and 33), and in 29, 30 and 31 among the
    closed ones.
- **Mis-hits at level 3 when inference was unrestricted:** "Adoption criterion" became
  migration, and "Local execution, build closure and qualification" and "Investigation
  dispositions and remaining acceptance" became verification. Restricting inference to level 2
  removed these.
- **Remaining level-2 approximations:** "Foundations and target design" becomes baseline, and
  "Confirmed rule changes and adoption routes" becomes migration.
- **Conclusion.** Legacy inference is good for checkpoint, packets and dispositions, and only
  approximate for the design-part kinds.

**Relation to existing owners (*Interface-checked*).**

- The repository already has a fence-aware heading scanner, `scripts/adr.py::markdown_headings`,
  and markdown-it table parsing in `scripts/document_lifecycle.py`. A production CLI would share
  one parser owner, not add a third.
- The read-only FastMCP precedent (`scripts/library_catalog_mcp.py`) shows an MCP face can sit
  over the same core. A CLI is still needed for Codex reach (prior `library-alternatives.md` §E).
- `.venv-docs` already has markdown-it-py 4.2.0. **mdit-py-plugins** is needed only for
  `{…}` attribute parsing and front matter. Comment markers need no plugin, and
  `document_metadata.py` already reads front matter.

---

## (c) Pairing each section kind with guidance, without duplicating the skills

These options were compared on scratch copies of `plan-structure.md`, the `execute-plan`,
`create-plan` and `plan-execution` skills.

| Option | Duplication | Granularity | Breakage when skills change | Result |
|---|---|---|---|---|
| **Part table row in `plan-structure.md`** (Part → "What it establishes") | none: the CLI reads the existing table | one line per kind | renaming a Part needs the kind→Part map updated, or a `Kind` column in that table | *Tested*: `checkpoint` returned `plan-structure.md:23 \| Finding disposition and current state: Traceability to source findings, open decisions, deferred work and the next executable step` |
| **Inverse markers in the skills**: `<!-- guides: checkpoint, dispositions -->` above a paragraph | none: the paragraph stays where it is; the CLI quotes it with `file:line` | paragraph | none from heading renames, because the marker moves with its paragraph; unknown kinds are simply unused | *Tested*: five markers placed in the copies. `section … checkpoint --guide` returned four paragraphs: `execute-plan.md:63` (reporting and closing findings), `:71` (update the existing checkpoint, no parallel ledger), `plan-structure.md:143` and `:149` (concise state, decisions and next steps) |
| Anchor references from a kind map to `skill.md#heading` | none | whole section, so coarse (`#transitions-evidence-and-current-state` is about 16 lines) | a heading rename breaks the reference silently; the fix is not mechanical | not run; follows from the same section parser |
| Schema `description` / `markdownDescription` | **a second copy** of the guidance | per field | drift | rejected on the single-declaration rule |
| Extraction by keyword search | none | noisy | — | not pursued |

**Interpretation.**

- The two zero-duplication options compose: the Part row gives a one-line "what this section
  establishes", and the inverse markers add the paragraph-level "what to consider".
- The guidance owner, the skill, declares which kinds it serves. No reference points into the
  skill from outside, so nothing can dangle.
- **Cost:** a handful of one-line comments in tracked process skills. Those are visible to raw
  readers and invisible in mdBook.
- The kind vocabulary can be declared once, as a `Kind` column or marker in the
  `plan-structure.md` Part table. It stays open: an unknown kind gets no guidance, never an
  error.

---

## (d) Migration "going forward"

- **No conversion is needed to start.**
  - Legacy inference finds a checkpoint in every active plan and packets or dispositions where
    such a heading exists (see (b)).
  - Plans without markers stay readable and queryable as they are.
  - Unmarked headings and unknown kinds are not findings. No lint is involved.
- **Mechanical conversion on demand (*Tested*).** `probes/plan-query/convert.py` (14 lines)
  appends `<!-- kind: X -->` to each level-2 heading whose legacy alias matched.
  - On a copy of Plan 33 it changed **exactly 8 lines**, all of them heading lines
    (`results/convert-33.diff`).
  - Re-parsing reports `via inline-comment` for each.
  - mdBook 0.5.4 `<main>` before and after: **all `id` attributes are identical**. With HTML
    comments stripped, the only difference is a trailing space inside eight heading texts.
  - Inbound `#anchor` links are therefore unaffected in the published book.
- **New plans** get the comment markers when the plan is created. The create-plan skill's
  outline or a scaffold can carry them pre-filled, so the agent types nothing extra. Existing
  plans can be converted when someone next edits them, or never.
- **Not established.**
  - The GitHub file view's anchor slug for a heading with an inline comment. No remote rendering
    was done; GFM sanitizes comments, so it is likely unchanged, but this is only
    *Interface-checked* by inference.
  - Marksman and the VS Code Markdown language service with these markers. Not run, because
    Q1 was dropped.

---

## Most promising options

| Option | Agent stops | Agent newly does | Programmatic action | Maturity / versions | Cost |
|---|---|---|---|---|---|
| `<!-- kind: X -->` on level-2 headings, kinds declared once in the `plan-structure.md` Part table | guessing which heading holds the state; scanning whole plans for where to update | nothing for new plans if the outline or scaffold carries the markers; nothing for legacy plans | legacy-title inference; the 14-line converter on demand | CommonMark raw HTML; mdBook 0.5.4 id-neutral (*Tested*) | one comment per kind section |
| Parse-on-demand plan query CLI (`outline` / `section --own --guide` / `rows --where` / `elements` / `mentions` / `active`) | reading 51–159 KB plans to resume; hand-maintained "status" restatements | runs one command; supplies its own filter regex | markdown-it-py line maps; verbatim numbered source lines usable as `Edit` targets | markdown-it-py 4.2.0 (already in `.venv-docs`), with mdit-py-plugins 0.6.1 only for `{…}`; resume bundles were 10–19% of the plan (*Measured*) | about 300 lines, sharing the parser with `document_lifecycle.py`; an optional MCP face per the catalog precedent |
| Inverse `<!-- guides: kinds -->` markers in process skills, plus the Part-table row | rereading 150–220-line skills to find the paragraph that applies to the section being edited | nothing (maintainer-placed markers) | the CLI quotes the marked paragraphs with `file:line` | *Tested* on copies | about 5–10 one-line comments in tracked skills |
| Heading per element with a `- State:` line, for concurrently edited elements | resolving merge conflicts on adjacent table rows | writes the element as a heading instead of a row, where concurrency matters | the `elements` query gives a table-like view back | mdBook native; 0 conflicts in S-a/S-b/S-c (*Tested*) | none |
| mdq 0.10.0 / mq 0.9.2 | — (read-only viewing only) | — | — | stable binaries; prose byte-faithful, tables re-rendered (*Tested*) | a binary download; no kinds, no line numbers |
| YAML / TOML / JSON plan encodings | — | indentation, escaping or table-ordering discipline | ruamel/tomlkit 1-line edits; yq `line` | *Tested* hazards above | projection for mdBook; silent misparse risks (YAML key, TOML table binding) |

## Material uncertainties

1. **Kind vocabulary granularity.** The kinds probed follow `plan-structure.md`'s eight Parts plus
   checkpoint and outcome. Whether the checkpoint and history split should be a kind
   (`history` or `handoff`) or left to subsection position is a content choice.
   `--own` already uses subsection position.
2. **State predicates are agent-supplied regexes.** That honours "tools gather, agents
   interpret". The filters used here (`^(Integrated|Implemented|Done|Complete)`, `^resolved`)
   matched the current wording of Plans 28 and 33, and will miss rows whose wording differs.
3. **ID collisions across plans** (prior exhibit X3) are not solved by in-file `mentions`. A
   cross-plan query would need plan-qualified IDs or file scoping.
4. **The probe reads copies taken at about 22:00 on 2026-10-09.** Live plans may since have changed.

## Retained artifacts

The runs used a session scratch directory. Retained here under `probes/plan-query/`: the query
CLI probe `planq.py`, the on-demand marker converter `convert.py`, the four-format generator
`formats_gen.py`, and the nine run outputs in `results/` (including `convert-33.diff`). Not retained:
the corpus copies (taken from `docs/plans/*.md` and four process-skill files at about 22:00 on
2026-10-09, with five `guides:` markers added to the skill copies only), the generated format
files and merge scenarios, the mdBook fixture and built output, the mdq/mq/yq binaries and the
scratch venv. The probe scripts read copies, so rerunning them needs a copied corpus.
