# Documentation system design review

## 1. Scope, drivers and coverage

Design tier, target purpose; documentation publication, architectural context and citation
migration. Core 3.0 and pse-arrow binding apply. Process-simulator profile 1.1 is not applicable:
no physical model, numerical contract or scientific result changes. Author review by Codex,
2026-09-25; not an independent review or a design-review pilot.

Inspected: ADR-0033/0036, proposed ADR-0095, Plan 18, publishing configuration and adapter,
ADR resolver, shared rules and existing book/CI boundaries. This review accepts the design
argument; [Plan 18](../../plans/18-architectural-documentation.md) owns executed qualification.

## 2. Decomposition and dependencies

Authored Markdown owns contracts and explanations. Site configuration owns collection selection
and tool versions. Discovery/staging composes mdBook rendering with Pagefind indexing; only
the adapter knows their output boundary. ADR tooling owns section identity. Browser components
own search interaction. The product and its native environment are not dependencies.

Pure discovery and identity fixtures need files only; publication integration needs the two
static binaries. A renderer upgrade affects the adapter and its fixture, not authored contracts.

## 3. Contracts, authority and constraints

A numbered section has one normative owner across blueprint and extracted pages. Old anchors
link to that owner; no second section registry is maintained. The blueprint owns revision history.
ADR metadata owns recorded decision status. Plans/packets own progress. The publishing declaration
selects current reading groups because old plan statuses alone do not identify current work.
The section directory, SUMMARY and search index are disposable derived output.

## 4. Change scenarios

Plan 18 S1–S6 define the cases: add a document, move a section, scope search, change a function,
build without product infrastructure, and fail a rebuild. File discovery absorbs ordinary
new pages; contract owners absorb semantic changes; output replacement owns failure recovery.
The remaining mixed blueprint is Reference, with current entrypoints linking its contract
precedence sections. This is a declared migration boundary, not inferred semantic certification.

## 5. Execution and lifecycle

Stage within ignored build output, render, annotate canonical chapter bodies, index, then
replace the local artifact. A failed render/index cannot replace the last successful artifact.
Optional same-commit rustdoc is copied separately and is outside the document search index.
Scope filtering defaults to Current. Everything clears that filter. Ordinary HTML navigation
works without JavaScript; no hosted search or custom application server is required.

## 6. Architectural assessment and gates

| Foundation | Argument / scenario | Design verdict |
|---|---|---|
| AP-01 | Authoring, navigation, identity and search have separate owners; S1/S2 | satisfied |
| AP-02 | Markdown/section IDs remain stable across rendering and moves; S2 | satisfied |
| AP-03 | Discovery → rendering → indexing composes existing tools; S1/S5 | satisfied |
| AP-04 | Derived navigation and section directory share the existing owners; S1/S2 | satisfied |
| AP-05 | Collection selection, tool versions and publication effects are explicit; S3/S6 | satisfied |
| AP-06 | Focused source context and file-only unit fixtures avoid product dependencies; S4/S5 | satisfied |

| Gate | Design assessment |
|---|---|
| G1 Authority | Pass: one numbered owner, derived publication, historical evidence preserved |
| G2 Semantic fidelity | Pass: migration retains identifiers; scope is a reading purpose, not correctness |
| G3 Validity | Pass: missing titles/owners and duplicate paths/IDs fail mechanically |
| G4 Hidden behavior | Pass: declared roots and independent docs commands; no product execution |
| G5 Recovery | Pass: publication follows successful rendering/indexing, restores failed replacement |
| G6 Transformation/reuse | Pass: full derived rebuild removes stale pages; no semantic cache |
| G7 Truthful claims | Pass: author review is Proposed design reasoning; Plan 18 owns execution claims |
| G8 Library use | Pass: mdBook/Pagefind own rendering/search; no bespoke search engine |
| G9 Architectural fitness | Pass at design level, supported by the six arguments above |

## 7. Findings

No unresolved design-level MUST gap in the stated target. Implementation qualification remains
with Plan 18 and cannot be inferred from this review. Existing single-file/manual-navigation
requirements conflict with the target and are explicitly routed below.

## 8. Library fit

mdBook retains existing Markdown rendering and URL conventions. Pagefind provides a static CLI,
index assets and component UI. The adapter owns version-sensitive HTML annotations and source
staging; thin browser integration owns scope selection. No library types enter production code.
Upgrade cost is bounded by the publisher fixture and real search check; pinning is warranted
because those concrete interfaces are consumed.

## 9. Alternatives

Raising the old index warning threshold leaves loading and retrieval problems. Excluding all
history would lose useful context. Replacing the generator would expand migration without
settling ownership. The selected composition retains the current renderer and makes history
explicitly searchable. A custom knowledge/proof graph adds a competing maintenance obligation
and is excluded. Further blueprint extraction follows substantive subsystem changes.

## 10. Verification

Proposed architecture is supported by the contracts above. Plan 18 records actual publisher,
identity, links, browser and tooling checks. No numerical performance or process-effectiveness
claim is made. A single browser/network observation is sufficient for the scoped search check;
no permanent ranking benchmark or proof archive is justified.

## 11. Authority changes and disposition

ADR-0095 documents intended supersession of ADR-0033/0036. Their bodies remain historical;
formal status transitions use the existing decision PR. Blueprint revision 54 records the
maintainer-authorized extraction. `PSE_DESIGN_EDIT=1` is used because changes to the authoritative
collection are the explicitly approved work. Plan 18 owns rollout and any discovered defects.

## 12. Decision

Architectural fitness: Accept at Proposed design level. Behavioral adequacy: Accept the
specified publishing contracts; implemented behavior requires the linked execution checks.
Overall: Accept the target design within this documentation-only scope. This author review
establishes neither independent review, pilot effectiveness, product qualification nor formal
ADR acceptance.
