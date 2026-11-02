---
title: "Design import coverage and verification"
---

# Design import coverage and verification

Imported from the connected Notion `pse-arrow Design` workspace on 2026-11-02. The [source map](source-map.md) gives the exact local destination for every narrative page and record.

## Coverage

| Source population | Retrieved | Local destination |
| --- | --- | --- |
| Narrative pages | 41 | Simulation, process, delivery and rationale chapters, including seven layer pages and six worked examples |
| General simulation concepts | 100 | [Simulation catalog](catalog/simulation-concepts.md), grouped by nine layers |
| Process concepts | 51 | [Process catalog](catalog/process-concepts.md), grouped by seven areas |
| Original domain acceptance workflows | 12 | [Full workflow records](reference/acceptance-workflows.md) |
| Answer kinds | 5 | [Answer kinds](reference/answer-kinds.md) |
| Task classes | 30 | [Task classes](reference/task-classes.md) |
| Canonical forms | 16 | [Canonical forms](reference/canonical-forms.md) |
| Kernel kinds | 5 | [Kernel kinds](reference/kernel-kinds.md) |
| Design decisions | 24 | [Decision properties and resolution narratives](rationale/design-decisions.md) |
| Version records | 6 | [Version properties, relations and revision narratives](rationale/version-records.md) |
| Total database records | 249 | Every record has an original source identity and a local anchor |

All nine data-source queries returned `has_more: false`. Each of the 249 record pages was also fetched so narrative resolutions and notes outside database properties were captured. All 41 narrative pages and their native child pages were fetched; there were no remaining unfetched child pages or unmapped internal Notion references. No truncation or unsupported-block notices were returned in those responses.

The [notebook register](reference/notebook-families.md) retains families 1–55 in sequence, the separately held family, all 20 archived path entries, the two developer entries, the zero-cell dummy exclusion and required non-notebook coverage. These retain their original inspection and execution distinctions.

## Conversion and preservation

- The original scientific prose, tables, examples, pins, source citations and catalog fields are retained. Record properties and their page bodies appear together.
- Internal page mentions, database views and relationship references resolve to local files or record anchors. Original source URLs remain available beside each imported page or record and in the source map.
- Notion formatting becomes ordinary Markdown: bold, inline code, dates, callouts, child-page links and database links. The scientific examples' three fenced code blocks were checked against their source text. Literal indexed concentration expressions are escaped to prevent Markdown from interpreting them as hyperlinks.
- All six version records retain their properties. The superseded overall-domain v1 and v2 full drafting documents remain linked at their Notion sources rather than copied as duplicate design chapters. Their decisions, current v3 replacement and version relationships are present locally. The superseded process-v1 adoption record is retained in full because it records the original scope/evidence basis.
- Source dates and original inspected revisions remain intact. The handoff separately records the published-main/local-checkout difference observed during this import.

## Checks performed

| Check | Result |
| --- | --- |
| Narrative and database-record source-to-destination coverage | 41 narrative pages and 249 records accounted for |
| Catalog fields and nonempty record notes | Preserved, with only the two explicitly linked superseded draft bodies excluded |
| Scientific prose/value token preservation | No missing source words or numeric tokens detected in the imported narrative or record sections |
| Rich-text link identity | Imported page/mention/database edges checked against their original source targets |
| Local file and anchor resolution | No broken local targets detected |
| Notebook family numbering and archive population | 1–55 complete; 20 archived entries retained |
| Fenced examples, Notion block conversion and whitespace | Passed |
| Repository `just turn-end` | ADR index and formatting completed successfully |
| Existing uncommitted files | All 15 preexisting modified files unchanged |

This collection contains 71 Markdown files, including its reading guide, strategy, handoff, source map and this verification report. Verification here concerns documentation completeness and conversion; simulator implementation and scientific runs remain with their existing evidence owners.

The source is ready for repository agents. Website publication requires the collection selection described in the [organization strategy](import-strategy.md#publication-and-maintenance-follow-up).
