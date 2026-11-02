---
title: "Documentation import and organization strategy"
---

# Documentation import and organization strategy

Keep the detailed design together in `docs/process-simulator-design/`, with a short planning entrypoint and explicit links to the repository's existing owners. This gives the implementation planner the complete semantic context while letting the architecture, ADRs and active plans continue to do their existing jobs.

The [source map](source-map.md) lists the exact destination of every narrative page and database record. The [import verification](import-verification.md) records coverage and the two deliberately linked historical drafts.

## Organization now in place

| Files or directory | Content and purpose |
| --- | --- |
| `index.md` | Reading order and the entrypoint for repository agents |
| `planning-handoff.md` | Focused scope brief for GPT-6.1 Sol to author its own implementation plan |
| `simulation/` | Overall domain v3, seven layer pages, ownership/reference rules, answer contracts, execution boundary, placement and six worked examples |
| `process/` | Process domain v1.1 and its five chapters, plus cross-industry semantic contracts |
| `delivery/` | High-level P0–P7 program, authoring, scientific packages, IDAES scope, qualification, implementation sequence, coding brief, native sourcing/catalog and industry expansion |
| `catalog/process/*.md` | All 51 process concept records grouped by seven areas, preserving each field and record narrative |
| `catalog/simulation/*.md` | All 100 general concepts grouped by nine layers, preserving ownership relationships and obligations |
| `reference/acceptance-workflows.md` | All 12 original domain witnesses, including full contracts, priority, source evidence, acceptance and notes |
| `reference/notebook-families.md` | All 55 numbered families, plus held, 20 archived, developer/dummy and beyond-notebook coverage |
| `reference/answer-kinds.md`, `task-classes.md`, `canonical-forms.md`, `kernel-kinds.md` | Five answer kinds, 30 task classes, 16 forms and five kernel kinds |
| `rationale/design-decisions.md` | All 24 Notion decision records and their resolution narratives |
| `rationale/version-records.md` | Six version records with history, relations and the live revision descriptions |
| `rationale/plan-detailing-assessment.md` | I01–I08 reasoning and incorporation map |
| `source-map.md`, `import-verification.md` | Source correspondence, completeness and maintenance facts |

Concept records use original Notion page identities as stable anchors. Grouped Markdown makes the full catalog searchable without forcing an agent to load every concept or duplicating definitions across a separate summary registry. The local catalog index pages link individual records.

## Authority and status

| Information | Owner |
| --- | --- |
| Agent conduct, permissions and operating workflow | `AGENTS.md`, applicable rules and process skills |
| Design assessment criteria | `docs/design_review/design_principles/standard.toml` and its selected core, profile, binding and companions |
| Enduring architectural contracts | Existing numbered owners under `docs/authoritative_design/sections/` |
| Decision rationale and formal acceptance | Repository ADRs through the established decision route; imported Notion decisions preserve the user's design choices and their original context |
| Domain detail and design intent imported here | This collection, including the original adopted versus proposed distinctions where they matter |
| Sequencing, scheduled work and current packet state | One owning implementation plan under `docs/plans/` and its native packet owners |
| Actual implemented behavior | Current source; implementation/test/measurement claims use their applicable evidence |

Do not turn a Notion `Decided` field into an accepted repository ADR, a catalog `Implementation` note into current qualification, or the P0–P3 witness priorities into P0–P7 program phases. Preserve those distinctions when summarizing them. Imported source citations at `9266555` and `19e7b49a` keep their original evidence meaning; refresh the relevant code before using them for a new implementation claim.

The native Rust/SurrealDB direction supersedes the earlier thermo-knowledge dependency. Detailed sourcing delivery choices remain with the [sourcing chapter](delivery/scientific-sourcing-and-catalog.md). Keep numerical hot-path work in prepared native data and use the database for the existing canonical/publication/discovery responsibilities; performance is a hypothesis until measured for the relevant workload.

## Integration with existing architectural owners

The following are suggested destinations for enduring contract changes as the plan resolves them. Reuse existing stable section identities; the import itself does not renumber or rewrite them.

| Imported source area | Existing repository destination |
| --- | --- |
| Overall scope, formulation responsibilities and one scientific authority | `architecture-overview.md` §0, §0.5–0.6, §1 and §2 D1–D14 |
| Material identity, basis/datum, scientific records, property/reaction semantics and external functions | `physical-semantics.md` §8–9 |
| Balance regions, unit composition, interfaces, topology, templates and package editing | `models-and-composition.md` §10–12 and §22 |
| Semantic identity, immutable scientific revisions, guarded source publication and canonical storage | `identity-and-publication.md` §5 and §20, especially §20.6 |
| Registry changes needed by concrete scientific/source/discovery consumers | `schema-and-relations.md` §4 and §6; change the registry and regenerate its outputs |
| Prepared scientific closure, capability selection, reuse and dependency invalidation | `mathematics-and-compilation.md` §7 and §14 |
| Numerical capability and initialization boundaries | `numerical-execution.md` §15–18 |
| Dynamic inventory, cases, Query roles, studies, fitting, observations, result coverage and explicit recording | `workflows-and-results.md` §13, §19 and §21 |
| Operator-facing inspection and engineering tooling | `native-host-and-engineering-tools.md` §21.7 |
| Original-space acceptance, diagnostics and qualification limits | `operations-and-validation.md` §23–24, especially §24.2 |
| Supported configurations, explicit refusals and remaining scientific scope | `scope-and-open-design.md` §25–26 and capability coverage |

Paths in the destination column are relative to `docs/authoritative_design/sections/`. Architectural changes follow the decision/design route and a revision row in `docs/authoritative_design/blueprint.md`. ADRs own rationale; the architecture owns the resulting contract. Accepted ADRs are superseded rather than rewritten.

The expanded IDAES target also requires reconciliation of `docs/relationship-to-idaes.md` and the touched principles binding descriptions. The published baseline still excludes examples, `models_extra`, applications and surrogates and mentions an older Python product/parity boundary. The imported [architecture chapter](delivery/architecture-and-process-authoring.md) identifies those conflicts. Scope the corrections against current code and accepted decisions; preserve clean-room behavioral reference use.

## Migration sequence

1. **Capture and organize the source.** Import narrative pages and rich database fields/record notes; preserve original identities, dates, decisions, source pins and limits. Convert internal links to local targets and retain exact source URLs. This is the collection now present.
2. **Reconcile the selected execution baseline.** Refresh GitHub and local source, current plans and relevant accepted ADRs. Classify each consequential mismatch as stale description, missing scientific content, missing consumer integration or a generic semantic/execution gap. Record material conclusions in the future owning plan rather than a second status ledger.
3. **Scope the implementation plan.** Use the [handoff](planning-handoff.md) to produce one coordinator with A–D first-delivery packets and a dependency-driven P2–P7 program. Freeze the finite target/configuration population in P0; do not equate notebook count with release equivalence.
4. **Route decisions and enduring contracts.** Carry adopted semantic contracts to existing numbered owners when their meaning and decision route are settled. Create only the ADRs required by repository rules; do not manufacture one ADR per imported decision or a new runtime type per concept.
5. **Implement through existing owners.** Extend authored packages first. Use source deltas, contextual admission, bounded immutable staging, guarded activation, research and conformance owners. A new adapter or generic capability needs an actual selected consumer.
6. **Maintain one account.** Update the local design owner when meaning changes; link current packet state from its owning plan. Resolve imported overlap by assigning one explanation owner and linking it, preserving source attribution. At closure, transfer enduring meaning and follow the repository's retention policy.

## Suggested implementation-plan files

Allocate `NN` from the live repository at authoring time. Published main had plans through 62; local Plan63 already exists. These names describe responsibilities, not preallocated identities or a requirement to split every topic.

| Suggested file under `docs/plans/` | Scope |
| --- | --- |
| `NN-flowsheet-process-domain.md` | Coordinator: target, baseline, decisions, P0–P7 dependencies, ownership and current packet state |
| `NNa-process-authoring-and-first-delivery.md` | A–D source/edit/inspect/execute/reference journey and acceptance |
| `NNb-targeted-scientific-content.md` | Case-driven scientific dependency sets, source policy, Rust intake, SurrealDB publication/discovery and reusable equipment |
| `NNc-idaes-scope-and-qualification.md` | Finite release/configuration population, non-notebook coverage, correspondence and qualification responsibilities |

Keep industry expansion context in this collection until a finite increment is selected. Other companions are justified only by a coherent target and meaningful independent contracts. Plans28, 32, 40, 53, 54, 58 and 58h keep their own status and outstanding obligations; Plan63/tooling progress does not qualify simulator science.

## Publication and maintenance follow-up

The source files are usable directly by repository agents. The new folder is not selected by the current `docs/site.toml`, so a normal book build will not publish it yet. If website publication is wanted, add a collection rooted at `process-simulator-design` with scope `Reference`, and a lifecycle collection default in `docs/lifecycle.toml` using the existing vocabulary, owned by `docs/process-simulator-design/index.md`. These existing configuration files were left outside this folder-only import.

When creating the implementation plan, maintain its lifecycle/native scope bindings and current-work publication selection under [the documentation guide](../dev/documentation.md#metadata-and-aggregate-scope). No recurring sync service, new skill, second scenario registry or new evidence framework is needed. Notion remains the original source reference; subsequent repository edits should preserve provenance and record substantive changes at their actual owner.
