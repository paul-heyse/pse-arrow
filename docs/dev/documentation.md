# Publishing documentation

Markdown is the canonical source. Start with the [architecture reading guide](../authoritative_design/sections/reading-guide.md)
and [current work](../plans/README.md); inspect production source only as needed for the claim.

## Commands

- `just bootstrap-docs` provisions the locked docs-only Python group in `.venv-docs`
  and installs the mdBook and Pagefind versions declared in `docs/site.toml`.
  It does not synchronize or compile the product Python environment.
- `just docs` discovers collections, stages sources, builds HTML and indexes canonical chapter
  content with Pagefind. Output is `docs/book/`; staging is temporary under `build/docs/`.
- `just docs-test` exercises publisher and citation fixtures without importing the product.
- `just docs-serve` builds and serves the complete static site. Run `just docs` after edits
  and refresh the browser. No custom watcher or service is needed.
- `just adr-index` regenerates the source-readable ADR index. Book navigation is generated
  during publication and has no committed SUMMARY to synchronize.

`just docs` establishes rendering/indexing. CI separately checks offline internal links and
fragments with lychee. External URL availability is not a publishing gate. Optional same-commit
rustdoc can be included under `docs/generated/rustdoc/`; its absence never starts compilation.
A failed render/index build preserves the last successful local site. A fresh successful build
removes obsolete pages and search assets.

## Add or change content

Add Markdown within an existing collection. Supply a scalar front-matter title or an H1;
existing metadata belongs to its existing owner. New collection roots and deliberate scope
exceptions are configured in `docs/site.toml`. Keep supporting assets inside the documented
collection and select their exact paths in its `assets` declaration in `docs/site.toml`.
Only selected assets are staged; retention alone does not publish an attachment. Do not hand-edit staged SUMMARY, derived section directories or generated schema docs.

Search defaults to Current: entry pages, the architecture sections, development guides, the
selected standard and active plans. The declaration selects current work explicitly because a
plan's lifecycle field is not a resume instruction. Accepted ADRs are Current; proposed ADRs,
library maps, generated schemas and the blueprint's revision/former-anchor page are Reference.
History holds only the few deliberately retained records outside those groups, such as a
closed plan kept as the highest-numbered record or a review whose findings are still open.
Retired material is not published at all, so Everything searches only what is retained.
Search engine/index data loads when the search dialog opens; ordinary page visits load only
the component UI. Scope describes reading purpose, not semantic validity.

To move an authoritative section, preserve its numbered heading in exactly one owner under
`authoritative_design/sections/` and remove the old body. Links use the publisher's stable
`#section-N-M` anchors, which follow the owner. When a cited mechanism is retired, keep its
identity as a one-line pointer under the page's "Retired section identities". Add the
collection revision row to `blueprint.md` through the normal design route. The shared ADR
resolver checks identifiers, not the truth of their prose.

## Lifecycle and retirement

Keep a document only while it is needed to understand, change, operate or qualify the
current system, or to complete active work (ADR-0096). When a plan closes, move its enduring
meaning to the owning section or ADR, remove it from `current_work`, and delete the completed
plan, its packets and resolved reviews once no reader, recipe, test or skill consumes them.
Keep the highest-numbered plan and ADR until a newer one exists. Before deleting, search for
inbound links and executable consumers (including constructed paths); repair links to the
current owner, or to an immutable permalink when a historical record is genuinely needed.
Git history is the archive: there is no archive tree, second book or retirement ledger.

Use relative links for published documents, repository links for source files, and immutable
Git links when citing a historical source version. Local-only capability references should be
labelled source paths rather than broken website links. Preserve the original conditions and
observations when repairing links in historical documents.

## Maintenance boundaries

A function-body edit normally needs no architecture update. Update a document when its enduring
contract, rationale or instructions change. Use relevant source and product tests for behavioral
claims. Do not add proof manifests, symbol inventories, source hashes, architectural scores or
routine full-product qualification to documentation work. Add mechanical automation only for a
concrete recurring defect whose prevention costs less than its upkeep.

The adapter consumes Markdown paths/metadata, the pinned mdBook HTML boundary and the Pagefind
CLI/Component UI. Tool upgrades exercise the same fixtures and a real build/search check.
Search assets are local, and ordinary chapter navigation remains usable without JavaScript.

## Metadata and aggregate scope

`scripts/document_metadata.py` is the shared interpretation boundary for publication,
ADRs and lifecycle queries. `docs/lifecycle.toml` declares versioned vocabulary, collection
and bundle defaults, exceptions and native scope bindings. Preserve native fields and body
bytes; controlled `doc_*` fields add role, topics, ownership and retention
meaning alongside them. Native scope bindings and sparse relationships describe dependencies. Unknown optional meaning remains unknown; contradictory definitions,
duplicate keys, invalid references and ambiguous scope bindings fail validation.
Canonical references are repository-relative. Legacy owner references beginning `docs/` are
also repository-relative; other legacy relative references resolve from their source parent.
Historical references do not invent a current owner. Immutable ADRs retain their original
bytes; an explicit path/key/scalar-digest adapter handles only declared historical encodings
before the same strict YAML parse. New and mutable documents use valid YAML.

```bash
scripts/pse-env --docs -- python3 -m scripts.document_lifecycle inventory --role plan
scripts/pse-env --docs -- python3 -m scripts.document_lifecycle validate
scripts/pse-env --docs -- python3 -m scripts.document_lifecycle scope --state open
scripts/pse-env --docs -- python3 -m scripts.document_lifecycle retire-plan --path 'docs/plans/32-*'
```

These are read-only JSON projections and retirement preflight. They neither authorize nor
perform deletion. Queries read the provisioned docs environment without syncing dependencies,
compiling the extension or scanning host storage. Use `just bootstrap-docs` explicitly when
that environment is missing.

Multiple plans may be active concurrently. Native packet/finding tables remain their status
owners; explicit bindings derive a scope view with document/binding/native-row identity,
raw state and source location. Missing state stays unknown. Relationships can point to another
content owner without copying its status; similar titles do not prove equivalent scope.
Update a binding when its native table changes. No second editable backlog is maintained.

For Plan 28 onward and future plans, maintain metadata/defaults and current-work selection
alongside authoring. This does not change the plan's substantive framing, implementation,
assessment, examples or presentation conventions. Closing one plan does not close another:
reconcile its dependencies and references, transfer enduring meaning, then apply the retirement
rules above. The specifically authorized pre-28 removal requires no salvage or scope migration.
