# Publishing documentation

Markdown is the canonical source. Start with the [architecture reading guide](../authoritative_design/sections/reading-guide.md)
and [current work](../plans/README.md); inspect production source only as needed for the claim.

## Commands

- `just bootstrap-docs` installs the mdBook and Pagefind versions declared in `docs/site.toml`.
  It uses cargo-binstall and does not synchronize or compile the product Python environment.
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
collection. Do not hand-edit staged SUMMARY, derived section directories or generated schema docs.

Search defaults to Current. The declaration selects current-work groups explicitly because old
plan lifecycle fields are not reliable resume instructions. Accepted ADRs are Current according
to their recorded status; proposed ADRs are Reference. Scope describes reading purpose, not
semantic validity. The remaining mixed blueprint stays in Reference while the architecture
entry page links its applicable contracts. History includes older work and observations.
Search engine/index data loads when the search dialog opens; ordinary page visits load only
the component UI. Everything removes the scope restriction.

To move an authoritative section, preserve its numbered heading in one owner under
`authoritative_design/sections/`. Leave the old anchor and a link at its previous location;
remove the old normative body. Add the collection revision row to the blueprint through the
normal design route. The shared ADR resolver checks identifiers, not the truth of their prose.

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
