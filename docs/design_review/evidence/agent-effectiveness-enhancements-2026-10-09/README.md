# Agent effectiveness enhancement evidence

Evidence for the [principal review](../../reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md)
at clean baseline `d0f2c41818a34539a910654dfea4760771603f45`, 2026-10-09.
This package supports a design judgment and candidate enhancements. It does
not implement those enhancements or qualify the scientific product.

| Evidence | Purpose and limits |
|---|---|
| [Host observations](host-observations.md) | Personal configuration, effective CLI/session exposure, persistent server status, live authenticated HTTP MCP and focused failure controls. |
| [Interface probe](probe-interfaces.py), [observations](interface-observations.json) | Reproducible read-only interfaces plus isolated mocked faults. Intentional parser refusals and the missing-context tool error remain visible. |
| [Activity retrospective](retrospective/retrospective.md) | Fixed event-time window, attribution, manually checked contextual leads, contrary evidence and parser limitations. Current review tree excluded. |
| [Retrospective metrics](retrospective/metrics.json), [audited leads](retrospective/audited-leads.json) | Descriptive aggregates and source pointers; no causal productivity or token claim. |
| [Retrospective extractor](retrospective/extract.py), [source manifest](retrospective/source-manifest.json), `retrospective/event-pointers.jsonl.gz` | Reproduction and content-free event pointers/hashes. Original session transcripts stay in the personal Codex logs; they are not copied here. |
| [Capability comparison](capabilities/capabilities.md), [version/source evidence](capabilities/version-source-evidence.md) | Context7 and exact-version primary source research; distinguish installed, configured, exposed and exercised capabilities. |

The sibling corpus-intelligence review supplied hypotheses and an assessment
method, not findings transferable to Rust/PSE. Existing Plan 29 evidence and
Plan 30 qualification remain historical under their own conditions. The
current-work owners continue to own their findings and campaign status.

Extraction and interface probes completed with native exit 0 against a zero
execution-failure baseline. The retrospective documents malformed source and
unrecoverable exits rather than treating them as successes. No full product
tests or performance campaign ran. Proposed evaluation tasks in the principal
review would supply new evidence if the maintainer selects implementation.

## Publication checks

`just turn-end` passed, exit 0 against the zero failure baseline: ADR index and
formatting both passed. Its existing assessment output is
`build/assessment/20261009T111639.261705Z-turn-end-254629-1b16dd/summary.md`.
This is the turn-end bundle, not product qualification.

Focused publication checks passed: 21 relative file links resolve, all five
finding anchors exist, evidence Python and JSON parse, and gzip event pointers
decode to 34,913 records. Independent aggregation of those pointers reproduced
108 sessions, six root trees, 24,377 launches, 16,204 launches with recoverable
native exits and 20,034 search/read classifications. Portable extractor `--help`
passed. These checks establish package integrity, not parser completeness or a
second full corpus extraction. Final Git inspection showed only this review,
its evidence package and the design-review navigation edit.
