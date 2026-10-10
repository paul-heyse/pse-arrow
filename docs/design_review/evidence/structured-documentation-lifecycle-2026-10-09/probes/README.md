# Probes P1 and P2 and the merge exhibit

**What was run.** These are the runs behind `../representation-exhibits.md`, following the
probe candidates in `../library-alternatives.md`. Only the small scripts, fixtures and result
files are retained here; the scratch venv, staged corpora, built sites and scenario directories
were not retained and are reproducible from the commands below. The two Markdown fixtures use
the `.markdown` extension so the publisher does not render them as pages; the probes run used
the same bytes under `.md` names.

- **Host:** Linux 7.0.0-38-generic x86_64. Runs on 2026-10-09/10.
- **Python:** CPython 3.14.7, uv-managed. The scratch venv `venv-p1` was created with
  `uv venv venv-p1 --python 3.14.7`.
- **Packages:** `uv pip install markdown-it-py==4.2.0 mdit-py-plugins==0.6.1 mdurl==0.1.2`.
  `uv pip freeze` showed exactly these three.
- **Tools:** mdBook v0.5.4 and pagefind 1.5.2, both from `~/.cargo/bin`. git 2.43.0.
- **Isolation:** every script ran with `python -I`.
- **Repository writes:** none. The P2 corpus was staged into this directory; a `find -newer`
  check over the checkout, excluding target/build/external/.git, found no files written.
- **Remote services:** none called during this assignment.

**Labels.** *Tested* means a script asserts or compares an outcome under the stated
conditions. *Measured* means an observed output that was recorded and not asserted.

## P1: typed containers in markdown-it-py

**Files.** `p1/fixture.markdown`, `p1/p1.py`, `p1/p1-output.json`, `p1/equal_length.py` and
`p1/equal-length-output.txt`.

**Command.** `venv-p1/bin/python -I p1/p1.py p1/fixture.markdown`, exit 0.

**Parser.** The `commonmark` preset plus `front_matter_plugin`, `attrs_block_plugin`,
`fieldlist_plugin` and one `container_plugin(name="typed")`. Its custom `validate` accepts a
first word in {finding, scenario, disposition}.

| # | Observation | Label |
|---|---|---|
| 1 | A family name in the fence info gives typed elements, with one generic plugin instance. `token.info` keeps the raw text after the fence, such as `" finding {#F-13 status=deferred owner=plan-33}"`. `mdit_py_plugins.attrs.parse.parse` on its `{…}` suffix returns `{id, status, owner}`. | Tested |
| 2 | Nesting by a longer outer fence works. `::::: finding` contains `:::: scenario {#S-01}`, whose parent resolves to F-12 through `SyntaxTreeNode`. Maps are absolute 0-based document lines. | Tested |
| 3 | `open.map = [open_line, close_line]`: the second value is the closing fence's own line index, and the close token has no map. An auto-closed container has `close.markup == ""` and its map ends at the line that closed its parent. | Measured |
| 4 | **Equal-length nesting silently misparses.** In `::: finding {#A}` … `::: scenario {#B}` … `:::` … `Tail.` … `:::`, the first `:::` closes the **outer** container. The inner one is auto-closed (`markup ''`), and `Tail.\n:::` becomes a top-level paragraph. There is no error. `close.markup == ""` is the only signal. | Tested (`equal-length-output.txt`) |
| 5 | A custom `validate` is also called with **empty params** for bare `:::` lines, through the lheading terminator check. A validate that indexes `split()[0]` raises `IndexError` during parsing. | Tested |
| 6 | An attrs line directly above an element (`{#F-12 status=open severity=high}`) attaches to the container open token, and `open.map[0]-1` is that line. **Stacked attrs lines separated by a blank line also attach**: F-14 got `id`, `status`, `class=extra` and `owner`, with stacked values merged. In that case `map[0]-1` is blank, so locating the line needs an upward scan. The attrs token itself is removed from the stream. | Tested |
| 7 | Bare attribute values cannot contain `#`. `finding=review:x#f02` raises `ParseError: Unexpected character whilst scanning bare value: #`; quoting the value works. Bare `:`, `-` and `.` were accepted. | Tested (`merge/gen.py` first run) |
| 8 | A field list (`:cause: …`, `:route: …`) as the first body lines of a container gives `field_list` tokens. `fieldlist_name_open.map` is an empty range `[n, n]`; the body map may include a trailing blank line. | Measured |
| 9 | Front matter on line 0 gives `front_matter` with `map [0, 4]`. | Measured |
| 10 | colon_fence registered **before** the container plugin takes every `:::` fence: 5 `colon_fence` tokens, 0 containers. Registered **after** it, containers win for all valid families: 6 containers, 1 colon_fence. | Tested |
| 11 | **Splice edits.** Changing `status=deferred` to `status=resolved` on line `open.map[0]` (attributes in the fence info), or `status=open` to `status=resolved` on line `open.map[0]-1` (attrs line), changes exactly one source line. Re-parsing gives the same token count, and the token stream differs only in that token's `info` or `attrs`. | Tested |
| 12 | **Control.** Without the container plugin there are no container tokens; the fences become paragraph text. The attrs lines then attach to the following **paragraph**: every `{…}` line attaches to whatever block follows. | Tested |

## P2: publishing route

**Files.** `p2/stage.py`, `p2/compare.py`, `p2/p2-step1-deflists.json`, `p2/routes/*`, plus
the staged trees and sites.

### Step 1: does the corpus depend on mdBook definition lists? (*Tested*)

**Staging.**
`cd /home/paul/pse-arrow && scripts/pse-env --docs --resource-class light -- python3 -B probes/p2/stage.py probes/p2/staged`
calls `scripts.docs.discover` and `scripts.docs.stage` on the live tree into scratch. It
staged **294 pages** from the dirty `main` at 4c24721e plus uncommitted work.

**Builds.**

- `mdbook build staged --dest-dir site-on` with `docs/book.toml` as staged.
- The same with `[output.html] definition-lists = false` (`site-off`).

**Comparison.** `compare.py` compared `<main>` per page.

**Result.** 298 HTML pages compared (294 chapters plus mdBook's own pages); **0 differ, and 0
contain `<dl>`**. On this baseline, turning definition lists off changes no published page.

### Steps 2–3: staging rewrite vs Python preprocessor, and anchors (*Tested*)

**Fixture.** `routes/fixture-body.markdown` is the P1 fixture without front matter and without the
equal-length case.

**Shared rewrite.** `routes/typed.py` converts each typed container into
`<div id class data-*>` with blank-line padding and a `**ID** title` label line. It consumes
attrs lines, scanning upward over blank lines, and writes a closing `</div>`.

| Route | Build | Result |
|---|---|---|
| A | rewrite in staging (`stage_route.py`) → mdBook, definition lists on (default) | see below |
| B | unchanged source + `[preprocessor.typed] command = "<venv python> -I typed_pre.py"`, `after = ["links"]` | `<main>` **byte-identical to A** (`cmp`) |
| C | no rewrite, definition lists on | garbled: dt/dd, with the attrs line as `<dt>` and fence lines as `<dd>` (as in D §0) |
| D | no rewrite, definition lists off | literal fence and attribute lines as paragraphs; bodies render |
| E | A + definition lists off | the field-list lines become one literal paragraph (`:cause: … :route: …`) |

**What route A shows.**

- Body Markdown renders inside the divs: strong text, an `<h2>` with its own mdBook id, a list,
  and the nested scenario div.
- Div ids `F-12`, `S-01`, `F-13` and `F-14` are present.
- **A field list inside an element still garbles under default definition lists.** The rewrite
  leaves `:cause:` lines in place. mdBook turns the label paragraph into
  `<dt id="f-13">**F-13**</dt>` with two `<dd>`. Field lists need converting, or definition
  lists must be off.
- Case-sensitivity is a practical hazard. `<dt id="f-13">` and `<div id="F-13">` coexist, and
  existing links use lowercase anchors (`#eff01`).

### Step 4, cheap variant: Pagefind (*Measured*)

`pagefind --site PF` ran on route A's output, with `data-pagefind-body` on `<main>` and
`data-pagefind-filter="status:open"` on the F-12 div. Inspecting the gzipped fragment
(`pagefind/fragment/*.pf_fragment`) showed two things:

- **Anchors include div ids**: `{"element":"div","id":"F-12","text":""}`, and likewise S-01,
  F-13 and F-14, alongside heading anchors.
- **The filter is recorded at page level**: `"filters":{"status":["open"]}` for
  `/fixture.html`.

Element-level filtering therefore needs either one page per element or custom records. I did
not test custom records or the query-time `sub_results` behaviour for div anchors, because
that needs a JS query harness.

## Merge exhibit (*Tested*)

**Files.** `merge/gen.py`, `gen2.py`, `merge-results*.json`, and the scenario directories.

**Setup.** One Plan 33 slice in four encodings, generated from the live rows: EFF00, EFF01,
EI01, EI06 and the F02 disposition.

**Merge.** `git merge-file -L ours -L base -L theirs merged base theirs` (git 2.43.0); the
exit code is the number of conflicts.

| Scenario | (i) baseline tables | (ii) simple tables | (iii) typed blocks | (iv) YAML |
|---|---|---|---|---|
| S1: EFF01 status edit ∥ new EI07 row at the end of the EI table | clean | clean | clean | clean |
| S2: new EI07 ∥ new EI08, both appended at the same place | 1 conflict | 1 conflict | 1 conflict | 1 conflict |
| S3: EFF00 status ∥ EFF01 status (adjacent elements) | **1 conflict** | **1 conflict** | clean, both edits kept | clean, both edits kept |
| S4: EFF01 progress ∥ EFF01 delivery text (same element, two fields) | **1 conflict** | **1 conflict** | clean, both edits kept | clean, both edits kept |

**Interpretation.**

- The difference comes from line layout. A table row is one physical line per element, so any
  two edits to the same or adjacent rows collide.
- Multi-line elements, whether typed blocks or YAML, separate their fields onto separate lines.
- S2 conflicts in every encoding, with a trivial resolution (keep both). A `merge=union`
  gitattribute would keep both automatically, but union is unsafe for status edits.
