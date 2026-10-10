# Library alternatives for a structured documentation system

**Role, baseline and sources.** This is library research for the formal design review of how
pse-arrow should author, validate, project and publish its reference content: reviews, plans,
ADRs, the deferred register, architecture sections, dev guides, evidence bundles and
instructions. It does not recommend an architecture. The baseline is the checkout on `main` at
4c24721e6, with uncommitted work present. Publishing runs mdBook 0.5.4 and Pagefind 1.5.2
through `scripts/docs.py` (staging, then HTML annotation). The isolated docs environment is
`.venv-docs` on CPython 3.14.7 with ruamel.yaml 0.19.1, markdown-it-py 4.2.0 and mdurl 0.1.2.

Facts come from four kinds of source:

- current PyPI, npm and crates.io metadata, fetched 2026-10-09;
- Context7 documentation for myst-parser, mystmd, sphinx-needs, StrictDoc, Doorstop, Markdoc,
  remark-directive, DVC, LinkML, Quarto and Pagefind;
- primary sources: the mdBook 0.5.4 tag's `crates/mdbook-markdown/src/lib.rs` and changelog,
  the pandoc changelog and MANUAL at 3.12.1, StrictDoc's release-notes `.sdoc` at 0.30.2, and
  the mdit-py-plugins 0.6.1 package source (container and attrs modules);
- a small set of runs on this host, all writing only to scratch.

**Evidence labels.** *Measured* means I ran it here, under the conditions stated. Everything
else is at most *Interface-checked*: documentation or source reading, with no run.

**Corrections to the brief's premises.**

1. Pydantic is in `uv.lock` at 2.13.5 through **idaes-pse 2.13.0** (the parity group), not
   through fastmcp. fastmcp is not in `uv.lock`. `.mcp.json` runs it ephemerally with
   `uv run --no-project --with fastmcp==4.0.5`; the current release is 4.1.0. Neither pydantic
   nor fastmcp is in `.venv-docs`.
2. mdBook 0.5.4 turns **definition lists on by default**, and `docs/book.toml` does not turn
   them off. Every colon-fence block syntax therefore degrades badly in the current book (§0).

---

## 0. Cross-cutting runs: how each candidate syntax degrades (*Measured*)

**Fixture.** One document, `render/src/variants.md`, holding 11 syntax variants of "a finding
with ID F-12, status open, a Markdown body".

**Renderers.**

- **mdBook:** the local mdBook v0.5.4 with `use-default-preprocessors = false`, run once with
  definition lists on (the default) and once with `[output.html] definition-lists = false`.
- **GitHub:** the REST endpoint `POST /markdown` with `mode=markdown`, which is GitHub's
  renderer for repository files. That endpoint did not post-process `> [!NOTE]`. GitHub
  documents alerts in its file view, so the alert row reflects the API output only.

| Variant | GitHub (file-mode API) | mdBook 0.5.4, default (def-lists on) | mdBook, def-lists off |
|---|---|---|---|
| V1 `{#F-12 …}` line + `::: finding Title` (mdit attrs_block + container), nested `::::` | literal text; body Markdown renders | **garbled**: `{…}` line becomes a `<dt>` with a generated id, `:: finding…` becomes a `<dd>`, closing fences become `<dd>::</dd>` | literal text paragraphs; body renders |
| V2 MyST `:::{finding}` + `:id:` / `:status:` lines | literal text; option lines join into one paragraph | **garbled** dt/dd | literal text; body renders |
| V3 MyST backtick ```` ```{finding} ```` | **code block**: body Markdown not rendered | code block | code block |
| V4 pandoc `::: {#F-12 .finding status=open}` | literal text; body renders | **garbled** | literal text; body renders |
| V5 remark `:::finding[Title]{#F-12 status=open}` | literal text; body renders | **garbled** | literal text; body renders |
| V6 Markdoc `{% finding id="F-12" %}` | literal text; body renders | literal text, with smart quotes rewriting `"` to curly quotes | same |
| V7 bare field list `:status: open` | literal text joined into one line | **garbled** dt/dd | literal |
| V9 `<!-- finding id=F-12 … -->` … `<!-- /finding -->` | **invisible**; body renders | invisible; body renders | same |
| V10 heading attributes `### Title {#F-12 .finding status=open}` | braces shown in the heading; anchor becomes `#finding-title-f-12-finding-statusopen` | `<h3 id="F-12" class="finding" status="open">`; anchor `#F-12` | same |
| V11 `> [!NOTE]` alert | plain blockquote (API) | styled admonition | styled |

**What these runs settle.**

- **Colon fences cannot reach mdBook untransformed while definition lists are on.**
  Padding the fences with blank lines does not help (runs W1 and W2 still produced dt/dd).
  Either the staging step or a preprocessor must rewrite typed blocks, or the build must set
  `definition-lists = false`. A search of `docs/**/*.md` for lines beginning `: ` or `:<letter>`
  found no current definition-list use (generated paths excluded; I did not check for
  multi-line definitions in other forms). On that limited search, turning definition lists
  off looks free for current content.
- **On GitHub, every directive syntax except V3 leaves the body as rendered Markdown.** The
  fence and attribute lines show as visible literal text.
- **Two forms are truly invisible on GitHub:** front matter (shown as a table on GitHub) and
  HTML comments.
- **Backtick-fenced MyST directives hide the body's formatting on GitHub and in mdBook.**
- **Heading attributes split anchors across renderers.** mdBook passes arbitrary `key=value`
  through as HTML attributes and uses the explicit id. GitHub prints the braces and derives a
  different slug, so a link written as `#F-12` resolves in the book but not on GitHub.

**Other runs.**

- **pandoc 3.1.3** (local apt build, which has no djot) parsed nested fenced divs with id,
  classes and key/value pairs. `commonmark_x+sourcepos` returned `data-pos` line:column ranges
  on each Div and wrapped every word in a positioned Span.
- **pandoc's Markdown writer is not byte-preserving.** It quoted the values, and rewrote the
  inner `::::` fence as `:::`, an equal-length nesting that other parsers close differently.
- **ruamel.yaml 0.19.1 round-trip.** Run in `.venv-docs` on a findings list with a `|`
  literal-block Markdown body, an inline comment, a quoted scalar, a flow list and a `>-`
  folded scalar:
  - With the default indent, a no-op load and dump **changed** sequence indentation
    throughout the file.
  - With `yaml.indent(mapping=2, sequence=4, offset=2)` and `preserve_quotes=True`, the no-op
    round-trip was byte-identical, and editing one `status` produced a one-line diff.
  - The body loaded as `LiteralScalarString`.

---

## A. Typed-block Markdown authoring

### A1. markdown-it-py 4.2.0 + mdit-py-plugins 0.6.1 (Python)

**Versions.** markdown-it-py 4.2.0 (2026-05-07, already pinned in the docs group).
mdit-py-plugins 0.6.1 (2026-05-13; requires `markdown-it-py>=2,<5`, so it is compatible with
4.2). This pair is exactly what myst-parser 5.1.0 requires (`markdown-it-py~=4.2`,
`mdit-py-plugins>=0.6.1,~=0.6`). *Interface-checked* (PyPI metadata).

**Capabilities.** *Interface-checked* from the 0.6.1 source unless noted.

- **`container_plugin(md, name, marker=":", validate=None, render=None)`.**
  - It emits `container_<name>_open` / `_close` and puts the raw text after the fence into
    `token.info`.
  - It sets `token.map = [startLine, nextLine]` and tokenizes the body with the same block
    state, so nested tokens carry absolute 0-based line maps.
  - A closing fence must be at least as long as the opening one, so nesting needs longer
    outer fences.
  - The default `validate` matches the first word of `info` against `name`. A custom
    `validate` can accept a family of names, or `{…}` attributes on the fence line.
  - One plugin instance is needed per container name, unless the custom validate
    generalizes.
  - The close token has no map. Measured in a pinned local index of this package, used as
    documentation.
- **`attrs_block_plugin`.**
  - A line consisting only of `{#id .class key=value}` (Djot/Pandoc attribute syntax,
    `%comments%`, stacking) attaches its attributes to the **next token**, which may be a
    container open.
  - The attribute token is then **popped**, so its source line is not on any token. Editing
    tools must infer it from the next block's `map[0]`; probe P1 tests this.
  - `allowed=` moves disallowed keys into `meta["insecure_attrs"]`.
  - `attrs_plugin` does the same for inline spans, images and code.
  - The attribute parser (`mdit_py_plugins.attrs.parse`) is a module function, so it can also
    parse attributes written in a container's `info`.
- **`colon_fence_plugin`.**
  - It emits **one** `colon_fence` token whose body is raw `content`, with no child tokens;
    the info string is e.g. `{note}`. A consumer must re-parse the body and offset its line
    maps. This is what MyST does in docutils.
  - When colon_fence and a `:` container are both registered, colon_fence takes `:::` if it
    is registered first. This conflict was measured in the same pinned index.
- **Other plugins.**
  - **field_list** (`:key: value` → `field_list_*` tokens), **deflist**, **front_matter**
    (one hidden token, line 0 only), **myst_blocks** (`(label)=` targets, `%` comments,
    `+++` breaks) and **myst_role** (`` {role}`x` ``) all exist and emit typed tokens with
    maps.
  - Inline tokens have no map. Only block tokens have one: `[start, end)`, 0-based, no
    columns.

**No byte-preserving round-trip renderer exists.**

- mdformat 1.0.0 (2025-10-16, `markdown-it-py<5`) renders tokens back to Markdown but
  normalizes the output.
- mdformat-myst 0.3.0 covers MyST directives, roles, targets, front matter and footnotes.
  I found no mdformat plugin for the generic `container` or `attrs_block` syntax; the search
  was PyPI names only and is not exhaustive.
- For agents this matters less than it sounds: agents edit with exact-string replacement, and
  a tool-made edit, such as a status transition, can be a line splice located by `Token.map`.

**Fit to the content model.** Good.

- It provides a typed element (container name), an ID and key/value attributes (attrs line or
  info), an arbitrary Markdown body, nesting through fence length, and absolute line maps.
- It imposes no fixed headings, because headings inside a body stay ordinary content.
- Relations would be attribute values or inline references parsed by our own code. Nothing in
  the library resolves cross-document references.

**Editing and readability for agents.**

- Plain text that is easy to edit by exact string.
- Fence length must increase with nesting depth, which is an easy mistake and is caught by the
  parse.
- Readability on GitHub: visible but harmless fence and attribute lines (§0). In mdBook, the
  syntax must be transformed or definition lists turned off (§0).

**Integration owner.** Python, in the existing `.venv-docs`: one small pure-Python
dependency. No JS or Haskell.

**Lifecycle and upgrade cost.** Low. These are the same maintainers as myst-parser, and the
two releases track each other.

**Uncertainty.** Still unmeasured:

- nesting containers with attributes;
- where the attrs line sits relative to the container map;
- field_list inside a container;
- equal-length nested fences (the pandoc writer's output);
- whether a splice edit re-parses identically.

Probe P1 covers all five.

### A2. MyST: myst-parser (Python) vs mystmd (JS)

**myst-parser 5.1.0** (2026-05-13, Python ≥3.11). *Interface-checked.*

- **Dependencies.** Hard dependencies on `sphinx>=8,<10`, `docutils>=0.20,<0.23`, jinja2,
  pyyaml, markdown-it-py ~=4.2 and mdit-py-plugins ~=0.6.
- **Use without running Sphinx.** `myst_parser.docutils_.Parser` with
  `docutils.core.publish_*`, the `myst-docutils-*` CLIs, and `create_md_parser()` (a
  configured markdown-it instance). Sphinx-specific roles and directives are then
  unavailable, and Sphinx still installs as a dependency.
- **Directive syntax.**
  - ```` ```{name} arg ```` or `:::{name}` (colon_fence extension).
  - Options as `:key: value` lines or a `---` YAML block.
  - `(label)=` targets, `:name:` labels, `{ref}` roles and `[](#label)` links.
- **Custom directives.** These are docutils `Directive` classes in Python. Body parsing goes
  through `nested_parse`, and docutils nodes carry source and line, which docutils sometimes
  only approximates.
- **Extensions.** attrs_inline, attrs_block, fieldlist, deflist, colon_fence and others.

**mystmd 1.11.0** (npm; 2026-09-21; Node CLI, with a thin pip wrapper). *Interface-checked.*

- **AST and cross-references.** Its own mdast-based AST (myst-spec) with unist `position`.
  Project-wide labels and cross-references are resolved in a POST phase, where a link becomes
  `crossReference`. External cross-references use `myst.xref.json`.
- **Directive options.** Three forms: `:key:` lines, a YAML block, or inline
  `{name .class #label key="value"}`.
- **Plugins.** JS plugins in `myst.yml` define directives with `arg`, `options` and `body`
  specs. `body: {type: 'myst'}` delivers already-parsed AST.
- **Outputs.** It builds its own site and exports (HTML, PDF/LaTeX, Typst, DOCX, JATS,
  Markdown through `myst-to-md` 1.0.17).
- **Frontmatter schema.** It has its own frontmatter schema (title, authors, …).

**Fit.**

- Both carry typed directives with ID, options and a Markdown body, plus labels and
  cross-references. mystmd adds a real project-wide reference resolver.
- myst-parser drags in Sphinx and docutils.
- mystmd is a second publishing stack, with a JS AST and a site theme. It would replace
  mdBook rather than feed it.
- Degradation: §0 V2 and V3. Colon-fence MyST reads acceptably on GitHub. The backtick form
  hides the body formatting.

**Integration owner.** myst-parser: Python, adding Sphinx and docutils to `.venv-docs`.
mystmd: Node 26.5.0, which is present on the host, as a new toolchain.

**Uncertainty.** Whether mystmd warns on unknown frontmatter keys, and whether it can run
headless to emit JSON AST without building a site, was not verified.

### A3. Pandoc fenced divs + Lua filters / panflute

**Versions.** pandoc 3.12.1 is the current release (2026-10-07); the host has the apt build
**3.1.3**. panflute 2.3.1 (2024-03-20, needs the pandoc binary). pypandoc 1.17.

**Capabilities.**

- **Syntax** (*Interface-checked*): `::: {#id .class key=val}` … `:::`, with attribute syntax
  as for fenced code. The `commonmark` reader forbids trailing colons after the attributes.
- **Parsing at 3.1.3** (*Measured*): nesting, id, classes and key/value pairs parse, and the
  `sourcepos` extension adds `data-pos` ranges. Since 3.11 the extension also works for djot.
- **Transformation:** Lua filters and panflute (Python, over JSON AST) transform the
  document.

**Fit.** Good attribute model, but:

- it needs a Haskell binary in the docs toolchain;
- the AST loses exact source except through `data-pos`;
- the writer normalizes (*Measured*), so round-trip is not byte-preserving.

Degradation: §0 V4.

**Integration owner.** The pandoc binary, plus Lua or Python filters.

**Lifecycle cost.** Moderate. The host's 3.1.3 is 11 minor versions behind. A pinned binary
would have to be managed like mdBook and Pagefind are in `docs/site.toml`.

### A4. Djot

**Implementations.**

- @djot/djot 0.3.2 (JS, 2024-12-19). The spec repository's last commit is 2026-07-01.
- jotdown 0.10.0 (Rust, 2026-04-21).
- pandoc reads and writes djot since 3.1.12, which the host's 3.1.3 lacks (*Measured*: not in
  `--list-input-formats`).
- **No maintained Python implementation:** PyPI `djot` 0.0.1 and `pydjot` 0.0.1 are
  placeholders. *Interface-checked.*

**Capabilities.** Attributes on every element, `:::` divs, and a cleaner grammar.

**Fit.** Poor for this repository. Djot is **not Markdown**:

- GitHub does not render djot;
- a djot file named `.md` would render as CommonMark, where emphasis differs (`*strong*` vs
  `_em_`);
- mdBook cannot read it.

It would require a full source-format migration and a non-Python parser. Its useful idea, the
attribute syntax, is already what `attrs_block` and pandoc borrow.

### A5. Markdoc

**Version.** @markdoc/markdoc 0.5.10 (2026-09-16), JS.

**Capabilities.** *Interface-checked.*

- `{% tag attr="v" %}…{% /tag %}`.
- A schema per tag declares attribute `type`, `required`, `matches`, `validate` and
  `description`.
- `Markdoc.validate(ast, config)` returns errors with `lines`.
- `Markdoc.format(ast)` pretty-prints back to source, normalizing it.

**Fit.** It has the best built-in "schema-validated attributes on typed blocks" of the
syntaxes considered, but:

- it is JS-only;
- the tag lines show literally on GitHub;
- mdBook's smart punctuation rewrites the quotes in tag lines (*Measured* V6);
- validating needs a Node step in the docs build.

### A6. remark-directive (generic directives proposal)

**Versions.** remark-directive 4.0.0, micromark-extension-directive 4.0.0 (2025-02-27),
mdast-util-directive 3.1.1 (2026-10-03). JS (unified).

**Capabilities.** *Interface-checked.*

- `:::name[label]{#id .class key=value}` containers, `::leaf` and `:text` directives.
- Nesting by longer outer fences.
- `containerDirective` nodes with `name`, `attributes` and unist `position`.
- Serialization back through mdast-util-to-markdown, which normalizes: fence length,
  attribute order (id, class, then the rest), quoting.

**Fit.** Similar to A1 but in JS. It has no schema layer of its own.

**Degradation.** §0 V5.

---

## B. Canonical data authoring (YAML/TOML with Markdown in block scalars)

**Versions.** ruamel.yaml 0.19.1 (pinned) and tomlkit 0.15.1 (TOML round-trip; not installed).

**Capabilities.**

- ruamel round-trip mode keeps comments, key order and block-scalar style
  (`LiteralScalarString`, chomping).
- With an indent configuration matching the file, a no-op round-trip and a single-scalar edit
  were byte-stable on the fixture (*Measured*, §0).
- Its default indent rewrote every sequence (*Measured*), so the indent must be fixed per file
  family or detected.

**Editing hazards for agents.** Interpretation, from YAML semantics:

- Every Markdown line in `|` must carry the block's indentation. An exact-string replacement
  that adds an under-indented line ends the scalar and creates a YAML error, or worse, a new
  key.
- Tabs are forbidden.
- Indented code blocks inside the body need the extra indentation on top.
- Long bodies make the meaning (Markdown) secondary to the container (YAML).

**Diff and merge.** Line-based, like Markdown. Reordering list items produces large diffs.
Moving content between bodies is noisier because of re-indentation.

**Rendering.** The body must be rendered to Markdown by our own projection. GitHub shows YAML
as a code view, not as prose.

**Fit.** Strong for small typed records: register rows, dispositions, packet status,
conditions. Weak for prose-heavy elements: findings, rationale, sections.

**Integration owner.** Python, already in the docs environment.

**Lifecycle cost.** Low.

---

## C. Traceability-as-docs tools

### C1. sphinx-needs 8.5.0 (2026-09-03; `sphinx>=7.4,<10`; jsonschema-rs)

*Interface-checked.*

- **Need types.** `needs_types` or `ubproject.toml [[needs.types]]` define directive, title and
  ID prefix.
- **Fields and links.**
  - `needs_fields` / `[needs.fields.*]` declare typed extra fields with descriptions and
    `schema.type` / `schema.enum`.
  - `needs_links` declares named link types with incoming and outgoing labels.
- **Schema validation.** `needs_schema_definitions` (since 6.0) is JSON-Schema-derived, with
  `local` and `network` validation and a severity per rule. It replaces `needs_warnings`.
- **Queries and views.** Filter strings are Python expressions used by needtable, needflow,
  needlist, need_count and others. Dynamic functions such as `copy()` are available.
- **Export and source tracking.** `needs.json` export through `build_json`. Each need records
  its docname and line.
- **MyST.** Supported through myst-parser directive syntax.

**Fit.**

- It models review → finding → disposition → packet chains directly: need types plus link
  types plus status.
- Its validation and reverse links are mature.
- It requires a **Sphinx build**, which means replacing mdBook or running a second build only
  to produce `needs.json`.
- It has no staleness concept beyond what is authored.
- Filter strings embed Python in content.

**Ceremony.** Moderate to high. The toolchain is heavy (Sphinx plus extensions). IDs are
mandatory if `id_required` is set.

**Integration owner.** Python, in `.venv-docs`.

### C2. StrictDoc 0.30.2 (released 2026-10-09; 32 direct requirements: fastapi, lark, docutils, `markdown-it-py==4.*`, html2pdf4doc, …)

*Interface-checked* from the release notes.

- **Formats.** The native format is SDoc, with a custom `[GRAMMAR]` per document: element
  types, fields, and relations (Parent, Child, File) with roles. A **Markdown format** has
  been maturing through 2026:
  - **0.26.0:** custom grammars gain `**Reverse role**` and `**Composite**`; field keys are
    harmonized to sentence case.
  - **0.27.1:** document config fields; section UIDs become linkable through `[LINK uid]`.
  - **0.28.2:** the reader validates sections.
  - **0.29–0.30:** PlantUML and Pygments support in Markdown.
- **Node encoding in Markdown.** A node is a heading followed by bold field lines with
  hard-break backslashes (`**UID**: REQ-3 \`), with the grammar referenced as
  `**Grammar**: file.gra.md`.
- **Exports.** html, html2pdf, markdown, rst, json, excel, reqif, sdoc, doxygen, spdx.
- **Web UI.** A FastAPI web UI editor.

**Fit.**

- Per-document grammars match "family-specific content model".
- The Markdown encoding **makes every node a heading**, an imposed structure, and puts
  metadata in visible bold lines.
- The pipeline is heavy and has its own publisher, so it would replace mdBook or run beside it.
- It has no staleness of evidence.
- Its Markdown syntax is young and still shifting release to release.

**Integration owner.** Python. Note that it pins markdown-it-py 4.x, which is compatible with
the docs pin.

### C3. Doorstop 3.2 (2026-07-10; Python <3.15; LGPLv3)

*Interface-checked.*

- **Storage.** One item per file, as YAML or Markdown with YAML front matter. A document is a
  directory with `.doorstop.yml`.
- **Item fields.** `active`, `derived`, `level`, `normative`, `ref`.
- **Fingerprints and suspect links.** Links are `- PARENT: <fingerprint>`, and `reviewed:`
  holds the item's own fingerprint. When a parent's content changes, the child's link becomes
  a **suspect link** until `doorstop clear`. `doorstop review` re-stamps an item.

**Fit.**

- The suspect-link mechanism is the closest existing analogue to "this answer is stale
  because something it rests on changed".
- Its granularity is item content only: not pins, toolchains or source revisions, unless those
  are themselves modelled as items.
- One file per item fragments prose documents such as reviews and plans.
- Review fingerprints add churn and ceremony.

### C summary

All three tools model typed IDs, links and status well. Each brings its own publisher or
format, which conflicts with keeping one Markdown authority and mdBook. Doorstop's
fingerprinted links are the one mechanism directly relevant to staleness.

---

## D. Schema layer

| Option | Version | Declares once with descriptions | Generates | Risk of second authority |
|---|---|---|---|---|
| JSON Schema 2020-12 + `jsonschema` | 4.26.0 (not installed) | yes (`description`, `enum`, `$defs`) | validation only; docs need a separate generator | low if the schema file is the only declaration of element families |
| Pydantic v2 | 2.13.5 in `uv.lock` via idaes-pse (current 2.14.0); not in `.venv-docs` | yes (`Field(description=…)`, `Literal`, validators) | `model_json_schema()` → JSON Schema 2020-12 | low; the Python class *is* the declaration; brings pydantic-core (compiled) into the docs environment |
| LinkML | 1.12.0 (2026-10-08; 28 direct requirements, antlr4 runtime, …) | yes (YAML: classes, slots, enums, descriptions) | `gen-json-schema`, `gen-pydantic`, `gen-doc` (Markdown per class, slot and enum), SQL and RDF | **medium-high**: a third language whose generated Pydantic/JSON Schema/docs must be regenerated and kept out of hand edits; heavy for roughly 10 families |
| sphinx-needs / StrictDoc grammars | (C) | yes, inside the tool's own config or grammar | the tool's own validation | ties the schema to that tool |

All facts in this table are *Interface-checked*.

**What the comparison settles.** Any of the first three can declare a family once.

- Pydantic or a hand-written JSON Schema keeps the declaration in one Python-side artifact.
- LinkML adds a generator layer, which the repository's codegen rule would then govern under
  "never edit generated".
- Whether the schema *describes* content or *dictates* it, for example required headings, is
  a design choice none of these tools makes for us.

---

## E. Projection and query

**Repository precedent.** *Interface-checked*: source read.

- `scripts/library_catalog_db.py` loads a small tracked JSONL into in-memory SQLite on every
  open: 157 records, "milliseconds, can never disagree with the file". It attaches a
  gitignored on-disk index (`build/library-usage.sqlite`) read-only when present.
- `scripts/library_catalog_mcp.py` is a read-only FastMCP stdio server whose tools carry
  `ToolAnnotations(read_only_hint=True, idempotent_hint=True)`. It reloads when the backing
  files change.
- `.mcp.json` registers it with `uv run --no-project --with fastmcp==4.0.5`. Note that the
  pinned version is outside `uv.lock`.

**Options.**

| Option | Version | Freshness | Reach |
|---|---|---|---|
| Parse on demand (CLI, Python) | n/a | always current | any agent with a shell (Claude and Codex) |
| SQLite index rebuilt from sources, as in the precedent | stdlib | current if rebuilt on open (cheap at this scale) | shell, or MCP |
| Read-only FastMCP server | 4.1.0 current; 4.0.5 used | as above | Claude through `.mcp.json`; Codex only when MCP is opted in, so a CLI must exist anyway |
| DuckDB over extracted JSON | 1.5.6 (no dependencies) | needs an extraction step | SQL from a shell; a new dependency |
| DataFusion | already in the workspace (Rust 55.1.0) | needs an extraction step | Rust-side only, unless the Python bindings are added; disproportionate here |
| Generated Markdown views | n/a | regeneration discipline (codegen-check precedent) | GitHub and mdBook; becomes "generated paths" governance |
| Pagefind filters and metadata | 1.5.2 | at publish | readers of the site only |

**Pagefind detail.** *Interface-checked.*

- `data-pagefind-filter` and `data-pagefind-meta` are **page-level**. `docs.py` already sets
  `scope:` that way.
- Sub-results are per heading anchor.
- Element-level filtering, such as "open findings", needs either one page per element or
  `add_custom_record(url=…#anchor, filters=…)` through the Pagefind Python/Node indexing API.
  The `pagefind` PyPI package is 1.5.2.

**What the comparison settles.**

- At this content scale, parse-on-demand or rebuild-on-open SQLite avoids a stale second store.
  That is the precedent's argument.
- An MCP server is a convenience layer over the same query core, not a requirement, because
  Codex reach requires a CLI regardless.

---

## F. Rendering owner

### mdBook preprocessor protocol, 0.5.x

*Interface-checked*: guide and changelog.

- **Invocation.** `mdbook` runs `<command> supports <renderer>`: exit 0 means supported.
  It then runs the command again with `[context, book]` JSON on stdin.
  - The context holds `root`, `config`, `renderer` and `mdbook_version`.
  - The book holds `items`, with `Chapter{name, content, number, sub_items, path,
    source_path, parent_names}`.
- **Output.** The preprocessor writes the modified Book JSON to stdout.
- **Configuration.** `command` (relative to the book root since 0.5), `renderers`, `before`,
  `after` and `optional` (default false, so a missing preprocessor is an error).
- **Changes in 0.5.** `Book::sections` was renamed to `items`; `supports_renderer` returns
  `Result<bool>`; types are `#[non_exhaustive]`.
- **Language and pinning.** Any executable works, Python included. The Rust crate
  `mdbook-preprocessor` 0.5.4 pins pulldown-cmark 0.13.4 as public API.
- **Markdown options.** mdBook 0.5.4's parser enables tables, footnotes, strikethrough,
  tasklists, heading attributes, smart punctuation (default on), definition lists (default
  on), and GFM alerts as admonitions (default on). YAML metadata blocks and math are **not**
  enabled.

### Transforming in `scripts/docs.py` staging

The existing owner already stages sources, derives navigation, and post-processes HTML for
Pagefind (`annotate`).

**What the comparison settles.**

- A preprocessor sees the same chapter text the staging step sees, with no line maps beyond
  the content. It adds a second process boundary and JSON schema coupling to mdBook's
  `Book` type.
- Staging already owns this and runs in the same Python environment as the parser, so typed
  blocks can be rewritten there to HTML `<div id=… class=…>` or to headings with attributes.
- The preprocessor route matters only if `mdbook serve` or bare `mdbook build` must render
  typed blocks without the staging script.

Either way, §0 shows colon-fence syntax must be transformed, or definition lists disabled,
before mdBook sees it.

---

## G. Evidence frameworks

**In-repo baseline.** Interface-checked: source read. These mechanisms are the comparison
point.

- **`scripts/validation_scope.input_identity(scope, snapshot, environment)`.** It projects
  file-content identities by a versioned scope definition (`INPUT_SCOPE_VERSION = 3`;
  `RUST_INPUTS` includes Cargo.toml/.lock, rust-toolchain.toml, pyproject.toml, uv.lock and
  `.cargo`) plus selected environment variables.
- **`scripts/validation_receipts.py`.**
  - It reuses an observation only when `inputs` are identical, and it authenticates origin
    reports and artifacts by sha256.
  - It records an explicit `applicability_transfers` entry with `changed_inputs` and a
    `reason` when inputs moved.
  - This is a content-hash staleness and reuse model with recorded conditions.
- **`scripts/test_resources.py`.** A fixture-lifetime and reference ledger.
- **`docs/capability-maps/evidence` + `just evidence-regen`.** Committed manifests and
  lockfiles, plus a script that re-runs the extractions and probes and rewrites the captured
  outputs. It provides a rerun entry point but no staleness check.
- **`docs/design_review/evidence/<topic>/`.** Ad hoc probe sources (`*.rs`) and raw outputs
  (`*.json`, `*.log`), with no common rerun entry point or recorded conditions.

| Framework | Version | Question identity | Recorded conditions | Staleness detection | Rerun entry | Shared harness | Ceremony and ops cost |
|---|---|---|---|---|---|---|---|
| DVC | 3.67.1 (2026-03-31; repository moved to treeverse/dvc, active 2026-10-05; 42 direct requirements) | stage name in `dvc.yaml` | deps (file/dir hashes in `dvc.lock`), `params` (keys in YAML/TOML/JSON/py files), outs | `dvc status` (changed deps, params or outs); run cache restores outputs | `dvc repro <stage>` | no; each stage is an arbitrary `cmd` | `.dvc/` directory, `dvc.lock`, heavy install. Local cache works with no remote. Whole-file deps such as `uv.lock` mark every stage stale on any lock move; params key paths cannot select one package inside `uv.lock`'s array of tables |
| Quarto freeze | 1.10.19 | document | none beyond the source file | `freeze: auto` re-executes **only when the source file changes**; not on pins, toolchain or environment | `quarto render` | no | a whole publishing system (pandoc-based); does not fit |
| insta (Rust) | 1.49.0, already pinned and used by pse-authoring snapshots | test + snapshot name | snapshot header (`source`, `expression`, `input_file`), plus optional `info` metadata that can carry serialized conditions | only by re-running; a diff on drift; pending `.snap.new` | `cargo insta test` / `review` | nextest/cargo test | low; already in the tree |
| syrupy (pytest) | 6.1.1 (2026-09-13; requires `pytest>=8`) | test node id + index | none by default | re-run only; reports unused snapshots | `pytest --snapshot-update` | pytest | low; a new test dependency |
| MLflow | 3.17.0 (20 direct requirements: Flask, alembic, docker, …) | run, experiment | params, metrics, tags, git commit, artifacts | none (a record store) | none intrinsic | no | high (tracking store and UI) |
| Sacred | 0.8.7 (2024-11-26; low activity) | experiment + run | host info, package versions, sources, git commit, config | none | the experiment's CLI | no | moderate; maintenance risk |
| pytest / nextest as probe harness | pytest 9.1.1 and nextest in the tree | node id / filterset | whatever fixtures capture | re-run; selection by changed inputs (`just affected`) | the existing recipes | yes | lowest; already the repository's harness |

All rows except the in-repo baseline are *Interface-checked*.

**What the comparison settles.**

- No framework supplies **question identity tied to prose answers** together with
  **condition-granular staleness**.
- DVC is closest on content-hash staleness, but only at file granularity, and it adds an
  external state store beside `input_identity`.
- Snapshot tools provide drift detection only by execution.
- Run trackers record conditions but never decide staleness.
- The in-repo `input_identity` + receipts model already implements identical-input reuse and
  reasoned transfer. Generalizing that core, which is a design question, overlaps most of
  what DVC would add.

**Not settled.** Whether DVC's `params` on TOML keys plus per-file deps gives acceptably fine
granularity in practice. Probe P3 is optional.

---

## H. External and Git-native state

These points are interpretation from documented behaviour; no run.

- **GitHub issues and projects.** Status lives outside the checkout and is not versioned with
  the commit that changes the content. Reading or querying it needs network and an API (gh or
  MCP). It is visible to both agents only through tools. Projects fields and sub-issues could
  hold status and relations, but they create a second authority and break offline and
  historical reads, such as "what was open at commit X". Suitable as an outward notification
  channel at most.
- **Git trailers.** These are immutable per commit (`git interpret-trailers`,
  `git log --format=%(trailers:key=…)`). They suit event records, for example "Resolves:
  F-12", from which a history can be derived. They cannot be corrected without new commits,
  are invisible in the rendered docs, and would make current status a fold over history.
- **Git notes.** `refs/notes/*` are not pushed or fetched by default and GitHub does not
  display them. They are easily lost and invisible to agents unless explicitly fetched.
  Unsuitable as authority.

---

## Comparison table

| Alternative | Version(s) | Typed element + ID + key/value + Markdown body | Nesting | Source positions | Byte-preserving edit support | GitHub raw view | mdBook 0.5.4 (default config) | Owner / environment | Lifecycle cost |
|---|---|---|---|---|---|---|---|---|---|
| mdit container + attrs_block (+ field_list) | md-it-py 4.2.0, mdit-py-plugins 0.6.1 | yes (name; attrs line or info) | yes (longer fence) | block lines, absolute `[s,e)`; attrs line popped | none built in; line-splice by map (mdformat normalizes) | literal fences, body renders (M) | **garbled** dt/dd unless transformed or def-lists off (M) | Python, `.venv-docs`, +1 small dependency | low |
| MyST colon fence (myst-parser) | 5.1.0 | yes (`:key:` / YAML options, labels) | yes | colon_fence body raw at token level; docutils lines | mdformat-myst normalizes | literal, body renders (M) | garbled (M) | Python + Sphinx + docutils | moderate |
| MyST backtick (any) | — | yes | yes | — | — | **code block hides body** (M) | code block (M) | — | — |
| mystmd | 1.11.0 | yes, with typed directive option specs | yes | unist positions | myst-to-md normalizes | as MyST | not applicable (own site) | Node | high (second stack) |
| Pandoc fenced divs | 3.12.1 (host 3.1.3) | yes | yes | `sourcepos` `data-pos` (M) | writer normalizes (M) | literal, body renders (M) | garbled (M) | Haskell binary + Lua/panflute | moderate |
| Djot | djot.js 0.3.2, jotdown 0.10.0, pandoc ≥3.1.12 | yes (attributes everywhere) | yes | via pandoc sourcepos | — | **not rendered as djot** | cannot read | JS/Rust/Haskell; no Python | high (format migration) |
| Markdoc | 0.5.10 | yes + **schema-typed attributes, validate with lines** | yes | lines | `format()` normalizes | literal tags (M) | literal; smart quotes alter tags (M) | Node | moderate-high |
| remark-directive | 4.0.0 / mdast-util-directive 3.1.1 | yes | yes | unist positions | to-markdown normalizes | literal, body renders (M) | garbled (M) | Node | moderate |
| HTML comment markers (not in the brief; for contrast) | n/a | ID and key/value in the comment; body is ordinary Markdown | by convention | needs our own parser (html_block tokens) | trivial | **invisible** (M) | invisible (M) | Python | low |
| Heading attributes | pulldown-cmark 0.13.4 (mdBook) | ID, class, key/value on headings only | heading levels | heading map | trivial | braces visible; **anchor differs** (M) | native attributes (M) | — | low |
| YAML/TOML records with Markdown scalars | ruamel 0.19.1, tomlkit 0.15.1 | yes (data-first) | yes | ruamel `.lc` line info | ruamel round-trip byte-stable with matched indent (M) | YAML code view | needs projection | Python | low |
| sphinx-needs | 8.5.0 | yes + link types + schema | yes | docname + line | — | per MyST/RST | replaced by Sphinx | Python + Sphinx | high |
| StrictDoc | 0.30.2 | yes (grammar per document), node = heading | sections | — | own editor | bold field lines readable | own publisher | Python (heavy) | high |
| Doorstop | 3.2 | item-per-file YAML/MD | levels | — | YAML | front matter + Markdown | own publisher | Python | moderate; fingerprint churn |
| JSON Schema / Pydantic / LinkML | 4.26.0 / 2.14.0 (2.13.5 locked) / 1.12.0 | schema layer only | — | — | — | — | — | Python | low / low / medium-high |
| DVC | 3.67.1 | evidence stages | — | — | — | — | — | Python (heavy) | high ops |
| insta / syrupy | 1.49.0 / 6.1.1 | snapshot evidence | — | — | — | — | — | Rust / pytest | low |

(M) = *Measured* in this assignment, under the conditions in §0. Other cells are
*Interface-checked*.

## Material uncertainties

1. **mdit-py-plugins behaviour on our shapes is unmeasured** (probe P1): nested containers
   with attributes, the attrs line position, equal-length nested fences, field_list inside a
   container, and splice-edit re-parse identity.
2. **The GitHub file view may differ from `POST /markdown`.** It did for alerts. All other
   rows are consistent with the GFM specification, where none of these syntaxes is defined.
3. **The definition-list search covered only one-line patterns.** Turning definition lists off
   in the book is likely free, but other content forms were not checked (probe P2).
4. **Not verified:** mystmd's frontmatter strictness and headless AST export, and StrictDoc's
   Markdown-format stability across releases.
5. **DVC parameter granularity on TOML keys was not run** (optional probe P3).

## Retained artifacts

The runs in §0 wrote only to a session scratch directory. Their fixtures, the fetched upstream
sources (StrictDoc release notes, mdBook 0.5.4 `mdbook-markdown/src/lib.rs`, the pandoc
changelog and manual) and the rendered outputs were not retained. The follow-up probes P1 and P2
and the merge exhibit are retained under `probes/` with their conditions in `probes/README.md`.
The GitHub `POST /markdown` call in §0 sent only the synthetic 11-variant fixture shown in that
section, no repository content; it deviated from the review plan's no-remote-rendering intent and
is disclosed here.
