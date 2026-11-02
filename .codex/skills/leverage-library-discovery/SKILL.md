---
name: leverage-library-discovery
description: Discover and prioritize promising library capabilities from repository objectives, current usage evidence and broad library knowledge; maintain concise contextual briefings before detailed opportunity assessment.
---

# Discover capabilities worth investigating

Help an engineer or agent see what a library can do, what the repository already uses,
and which additional or underused capabilities merit deeper review. This is an optional
library-led entry point alongside the codebase-led audit. It requires neither a `CBO`
nor a diagnosed defect, and does not select an implementation target.

Maintain `briefing.md` in the relevant account under
[library assessments](../../../docs/library_leverage/libraries/README.md).
Use the [discovery and evidence guide](../../../docs/library_leverage/references/discovery-and-evidence.md)
for evidence interpretation and the
[framework](../../../docs/library_leverage/library-leverage-framework.md) for handoffs.

## Build a useful picture

Establish the selected library family, repository objectives, current configuration and
the inspected source state. Read the relevant architectural owners and broad library
overviews. Detailed codebase review is unnecessary: representative reads should establish
what observed uses accomplish, without inventing needs from names or counts.

Use generated package, API and reference evidence when available. Inspect its coverage
before interpreting it. An absent artifact is unprepared, an unresolved join is unknown,
and no observed references is not repository-wide absence. Keep spelling-based search
leads separate from resolved observations. Follow promising functionality beyond APIs
already referenced by the repository: configuration, extension points, composition and
companion packages can offer the greatest value.

Use Context7 first for documentation lookup, then primary documentation, exact source
and upstream tests for consequential details. API extraction broadens the candidate set;
it cannot establish semantic capability completeness or behavioral guarantees. An overview
may describe functionality requiring different features, a platform or an upgrade.

## Explain and prioritize

Keep the briefing short enough to read before a deeper review. Connect the major capability
families to current usage, then explain the most promising research directions. For each
shortlisted direction, give a concrete mechanism, why it could matter to these objectives,
the conditions that could defeat its value, and the next discriminating question.

Judge priority qualitatively by potential improvement and uncertainty worth resolving.
High reference counts do not establish importance; unfamiliar APIs do not establish value.
Include useful compositions and reasons to retain the present approach. Do not force a
quota, score every API, or turn the shortlist into the only eligible candidates.

Keep exact reusable technical knowledge in capability type pages. Link to those pages
instead of repeating their contracts. Follow the
[technical research guide](../../../docs/library_leverage/references/technical-contracts.md)
when selected questions need implementation-level depth.

## Hand off and refresh

A briefing owns current usage interpretation and proposed research priorities. Its hypotheses
are not applicability conclusions, new `CBO`s, accepted adoption or execution status.
When a concrete responsibility merits inspection, use `leverage-codebase-audit`; investigate
fit with `leverage-library-assessment`, then compare complete options with
`leverage-opportunity-review` within the authorized scope. Focused work can still enter
any existing stage directly. A reusable technical type may be researched without inventing
a current codebase match.

Refresh the briefing in place when objectives, usage or the available capability set change.
Record the actual inspected baseline and evidence limits; preserve inherited research dates.
Explain changed priorities rather than silently treating generated diffs as recommendations.
Leave broad overviews repository-agnostic. Discovery changes research documents, not product
code, dependency selection or implementation authority.
